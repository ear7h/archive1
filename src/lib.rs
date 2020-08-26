#![allow(dead_code)]
#![feature(associated_type_bounds)]

use std::{
    io::Read,
};


#[derive(Debug)]
pub enum Error {
    Network,
    Io(io::Error),
    Other(Box<dyn std::error::Error>),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}


macro_rules! type_assert {
    ($t:ty,$v:expr) => {
        if false {
            let _ : &$t = &$v;
            panic!();
        } else {
            $v
        }
    }
}

pub trait Pipe {
    type In;
    type Out;

    fn pipe(&self, i: Self::In) -> Result<Self::Out, Error>;

    fn then<O2, P2>(self, p: P2) ->
        ThenPipe<Self, P2>
    where
        Self : Sized,
        P2 : Pipe<In = Self::Out, Out = O2>
    {
        type_assert!(
            dyn Pipe<In = Self::In, Out = O2>,
            ThenPipe{
                first : self,
                second : p,
            }
        )
    }

    fn zip<O1, P1, O2, P2> (self, f1 : P1, f2 : P2) ->
        ZipPipe<Self, P1, P2>
    where
        Self : Clone + Sized,//j:'static,
        Self::Out : Clone,
        P1 : Pipe<In = Self::Out, Out = O1>,
        P2 : Pipe<In = Self::Out, Out = O2>,
    {


        type_assert!(
            dyn Pipe<In = Self::In, Out = (O1, O2)>,
            ZipPipe{
                inner: self,
                first: f1,
                second: f2,
            }
        )
    }

    fn into_dyn(self) ->
        DynPipe<Self::Out, Self>
    where
        Self : Sized,
    {

        type_assert!(
            dyn Pipe<In = Self::In, Out = Self::Out>,
            DynPipe{
                inner: self,
                next: vec![],
            }
        )
    }
}

#[derive(Default)]
pub struct IdPipe<T>(pub std::marker::PhantomData<T>);

impl<T> IdPipe<T> {
    pub fn new() -> Self{
        Self(Default::default())
    }
}

impl<T> Pipe for IdPipe<T> {
    type In = T;
    type Out = T;


    fn pipe(&self, i: Self::In) -> Result<Self::Out, Error> {
        Ok(i)
    }
}

#[derive(Clone)]
pub struct ConstPipe<T>(pub T);


impl <T : Clone> Pipe for ConstPipe<T> {
    type In = ();
    type Out = T;

    fn pipe(&self, _i: Self::In) -> Result<Self::Out, Error> {
        Ok(self.0.clone())
    }
}


/// Type erasure of the Pipe and Out so that the behavior can
/// be changed at runtime
pub struct DynPipe<O, P>
{
    inner : P,
    next : Vec<Box<dyn Pipe<In = O, Out = O>>>
}

impl<I, O, P> DynPipe<O, P>
where
    P : Pipe<In = I, Out = O>
{
    fn if_then<Pn>(mut self, b: bool, pn: Pn) -> Self
    where
        Pn : Pipe<In = O, Out = O> + 'static
    {
        if b {
            self.next.push(Box::new(pn));
        }

        self
    }
}

impl<I, O, P>  Pipe for DynPipe<O, P>
where
    P : Pipe<In = I, Out = O>,
{
    type In = I;
    type Out = O;

    fn pipe(&self, i : I) -> Result<O, Error> {
        let mut prev = self.inner.pipe(i)?;
        for next in &self.next {
            prev = next.pipe(prev)?
        }

        Ok(prev)
    }
}


pub struct ThenPipe<P1, P2>
{
    first : P1,
    second : P2,
}

impl<I, O, P1, O2, P2> Pipe for ThenPipe<P1, P2>
where
    P1 : Pipe<In = I, Out = O>,
    P2 : Pipe<In = O, Out = O2>,
{
    type In = I;
    type Out = O2;

    fn pipe(&self, i: Self::In) -> Result<Self::Out, Error> {
        Ok(self.second.pipe(self.first.pipe(i)?)?)
    }
}

pub struct ZipPipe<P, P1, P2> {
    inner: P,
    first: P1,
    second: P2,
}

impl<I, O, P, O1, P1, O2, P2> Pipe
for ZipPipe<P, P1, P2>
where
    O : Clone,
    P : Pipe<In = I, Out = O> + Clone,
    P1 : Pipe<In = O, Out = O1>,
    P2 : Pipe<In = O, Out = O2>,
{

    type In = I;
    type Out = (O1, O2);

    fn pipe(&self, i: Self::In) -> Result<Self::Out, Error> {
        let o = self.inner.pipe(i)?;

        Ok(
            (
                self.first.pipe(o.clone())?,
                self.second.pipe(o.clone())?,
            )
        )
    }
}

use ureq;
use url::Url;
use std::{
    fs::{
        OpenOptions,
        create_dir_all,
    },
    io::{
        self,
    },
    path::{
        self,
        Path,
        PathBuf,
    },
};

pub struct HttpPipe();

impl Pipe for HttpPipe {
    type In = Url;
    type Out = ureq::Response;

    fn pipe(&self, i : Self::In) -> Result<Self::Out, Error> {
        let res = ureq::get(i.as_str()).call();

        match res.into_result() {
            Err(err) => Err(Error::Other(Box::new(err))),
            Ok(res) => Ok(res),
        }

    }
}

pub struct HttpToReaderPipe();

impl Pipe for HttpToReaderPipe {
    type In = ureq::Response;
    type Out = Box<dyn Read>;

    fn pipe(&self, i : Self::In) -> Result<Self::Out, Error> {
        Ok(Box::new(i.into_reader()))
    }
}

pub struct UrlToPathPipe();

impl Pipe for UrlToPathPipe {
    type In = Url;
    type Out = PathBuf;

    fn pipe(&self, i : Self::In) -> Result<Self::Out, Error> {
        let mut pb = PathBuf::new();
        pb.push(i.scheme());
        if let Some(host) = i.host_str() {
            pb.push(host);
        } else {
            panic!("not host! {}", i);
        }

        pb.push(Path::new(i.path()).strip_prefix("/").unwrap());

        if i.path().ends_with("/") {
            pb.push("index.html")
        }

        Ok(pb)
    }
}

pub struct FsPipe<R> {
    base_path : PathBuf,
    phantom : std::marker::PhantomData<R>,
}

impl<R> FsPipe<R> {
    pub fn new(s : &str) -> Result<Self, io::Error> {
        let mut pb= PathBuf::new();
        pb.push(s);

        create_dir_all(&pb)?;

        Ok(Self{
            base_path : pb,
            phantom : Default::default(),
        })
    }
}

impl<R : Read> Pipe for FsPipe<R> {
    type In = (PathBuf, R);
    type Out = ();

    fn pipe(&self, i : Self::In) -> Result<Self::Out, Error> {
        let (pb, mut r) = i;

        let mut pb1 = self.base_path.clone();
        for comp in pb.components() {
            match comp {
                path::Component::Normal(s) => {pb1.push(s);},
                path::Component::ParentDir => {pb1.pop();},
                _ => {}, // ignore . / C:\\
            };
        }

        create_dir_all(pb1.parent().unwrap())?;

        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(pb1);


        let mut f = match f {
            Ok(f) => f,
            Err(err) => return Err(Error::Io(err)),
        };

        io::copy(&mut r, &mut f)?;

        f.sync_all()?;

        Ok(())
    }
}

