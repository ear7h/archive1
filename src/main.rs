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

/*
TODO

fn main() {
    println!("Hello, world!");

    IterPipe::new(file_list)
        .then(HtmlCrawlPipe::new()) // Pipe<Url, Iter<Item = (ArtifactId, String)>
        .map()// Pipe<(), (ArtifactId, String)>
        .into_dyn()
        .if_then(limit.is_some(), StringLimitPipe::new(limit.unwrap())) // Pipe<T, T>
        .then(LogPipe::new()) // Pipe< T : Display, T>
        .then(SearchIndex::new()) // Pipe< T : Display, T>
        .then(FsSink::new())?;

    IterPipe::new(file_list) // Pipe<(), Iter<Item = String>>
        .map()
        .then(UrlPipe::new()) // Pipe<String, Url>
        .then(HttpPipe::new() // Pipe<Url, Read>
            .if_then(limit.is_some(), limit_pipe) // limit_pipe : Pipe<Box<dyn Read>, Box<dyn Read>>
        .then(RssPipe::new()) // Pipe<Read, RssPost>
        .then(LogPipe::new()) // Pipe<T : Display, T>
        .then(RssStorePipe::new()) // Pipe<RssPost, RssPost>, stores structured data
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
*/
