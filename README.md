# archive1

My archiving app, with a hidden "functional" etl framework.

In rust terms, the framework revolves around the `Pipe` trait
which provides default methods for composing said `Pipe`s (ex.
`zip`, `then`).

In functional terms, there's probably a monad in there.

## Example

The folowing example downloads a file and stores
it in the local file system:

```rust
use url::Url;

use archive1::{
    Pipe,
    ConstPipe,
    UrlToPathPipe,
    HttpPipe,
    HttpToReaderPipe,
    FsPipe,
};


fn main() {
    let u = Url::parse("https://example.com/").unwrap();

    let out = ConstPipe(u)
        .zip(
            UrlToPathPipe(),
            HttpPipe()
                .then(HttpToReaderPipe()))
        .then(FsPipe::new("my-archive").unwrap())
        .pipe(()).unwrap();

    println!("out {:?}", out);
}
```

This isn't very impresive, but it's readable and maintainable. For example,
adding support for cloud storage, structured data extraction, or logging
is as simple as writing trait implementations and adding them to the pipeline.

One particular question that drove me to write this is logging specifically.
With this pipeline logging can be added with minimum
intrusiveness, reasonable granularity, and content awareness. In english,
we can add something like log entry severity ("debug", "warn", etc.) without
having to dig into the functional items themselves.


Here's a more complicated example (that will hopefully work in the
future) for downloading rss feeds

```rust
fn main() {
    IterPipe::new(file_list) // Pipe<(), Iter<Item = String>>
        .map()
        .then(UrlPipe::new()) // Pipe<String, Url>
        .then(HttpPipe::new() // Pipe<Url, Read>
            .if_then(limit.is_some(), limit_pipe) // limit_pipe : Pipe<Box<dyn Read>, Box<dyn Read>>
        .then(RssPipe::new()) // Pipe<Read, RssPost>
        .then(LogPipe::new()) // Pipe<T : Display, T>
        .then(RssStorePipe::new()) // Pipe<RssPost, RssPost>, stores structured data in a db
        .then(RssMediaList::new()) // Pipe<RssPost, Iter<Item = Url>>
        .map() // -> Pipe<_, Url>
        .zip(
            UrlToPathPipe
            HttpPipe::new() // Pipe<Url, Read>
                .if_then(limit.is_some(), limit_pipe) // limit_pipe :
                                                      // Pipe<Box<dyn Read>, Box<dyn Read>>
        .then(
            FsSink::new("rss-posts").filter(Dedup::new())) // Dedup : Pipe<Box<dyn Read, dyn Read>
                                                           // filter() consumes self so that Dedup
                                                           // would have access to FsSink (and its
                                                           // non-Pipe methods) to read the fs
        .fetch(()).unwrap();

}
```

## TODO
* test `DynPipe`
* consider breaking the different methods into their own
    traits with blanket impls over `Pipe`
* consider changing the `pipe` param from `Self::In` to `&mut Self::In`
    - & ref would remove the need for `Clone` in `zip`
    - mut is needed for things like `Read`
    - counter-point: `Read` shouldn't be shared in a `zip`!
    - counter-point: `Clone` isn't that bad since `Copy` implies `Clone`
* `IterPipe` and `map`
* `filter` method
* implement `Pipe` for `Fn(A) -> B`
* `fork` method for spawning new threads
    - also `zip_fork` (2 new threads + join) and `iter_fork` (with a
        parameter for the number of threads to be used a thread pool).
    - `async` will not be used in this library since the domain is
        inherently synchrounous. `fork` is good enough.


