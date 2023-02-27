use std::io::{prelude::*, BufReader, BufWriter};
use std::fs::{File, self};
use std::sync::Arc;
use std::thread;
use zip::{self, ZipArchive};
use std::path::{Path, PathBuf};
use tiny_http::{Server, Response, Method, Header, Request};
use walkdir::{WalkDir, DirEntry};
use std::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
struct Author{
    id: usize,
    name :  String,
    path:  String,
    manga: Option<Vec<Manga>>,
}
impl Author{
    fn new(id: usize, name : String, path : String) -> Self{
        Self {id: id, name: name, path: path, manga:  None}
    }
}
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
struct Manga {
    id: usize,
    name : String, 
    path: String,
}

impl Manga{
    fn new(id: usize, name : String, path : String) -> Self{
        Self {id: id, name: name, path: path }
    }
}

fn main() {

    let server = Server::http("0.0.0.0:8000").unwrap();
    let server = Arc::new(server);
    let manga_directory_path = "E:/Weeb Stuff/Manga/Doujinshi/Fakku/";
    
    let mut authors = index_author(manga_directory_path);
    index_collection(&mut authors);
    for request in server.incoming_requests() {
        handle_request(request, &authors)
    }
    
}

fn handle_request(request: Request, collection: &Vec<Author>) {
    let method = request.method();
    println!("{:?} - {:?}", method, request.url());
    // let res = serde_json::to_string_pretty(&collection).unwrap();
    // request.respond(Response::from_string(&res));
    // return;
    let path = request.url().split("/").collect::<Vec<_>>();
    //println!("{path:?}");
    let author_index = match usize::from_str_radix(path[1], 10) {
        Ok(f) => f,
        Err(e) => return
    };
    let manga_index = match usize::from_str_radix(path[2], 10) {
        Ok(f) => f,
        Err(e) => return
    };
    let page_index = match usize::from_str_radix(path[3], 10) {
        Ok(f) => f,
        Err(e) => return
    };
    println!("index = {author_index} / {manga_index}");
   
    match method {
        Method::Get => {
            println!("Getting FIle for = {author_index} / {manga_index}");
            let author = collection.get(author_index);
            let _manga = author.unwrap().manga.as_ref().unwrap().get(manga_index);
            let content_type_header = Header::from_bytes("Content-Type", "image/jpeg")
                .expect("That we didn't put any garbage in the headers");
            let mut buffer = Vec::<u8>::new();
            extract_manga_file_to_buffer(&_manga.unwrap().path, &mut buffer, page_index);
            println!("Responding = {_manga:?}");

            // request.respond(Response::from_data(buffer).with_header(content_type_header));
            
        },
        _ => {}
    }
    
}


fn index_collection (authors :  &mut Vec<Author>){
    for author in authors.iter_mut() {
        let mangas = index_manga_from_author(&author.path);
        author.manga = Some(mangas);
    }
}

fn index_manga_from_author(path: &str) -> Vec<Manga>{
    let walkdir = WalkDir::new(path);
    let mangas = walkdir.into_iter()
    .filter(
        |m| 
        m.as_ref().unwrap().file_type().is_file() && 
        m.as_ref().unwrap().path().extension().unwrap() == "cbz")
    .map(|f| f.unwrap())
    .collect::<Vec<_>>();

    let z = mangas.iter().enumerate()
    .map(|x| {
        Manga::new(
            x.0,
            x.1.path().file_name().unwrap().to_str().unwrap().to_owned(),
            x.1.path().as_os_str().to_string_lossy().to_string(),
        ) 
    })
    .collect::<Vec<_>>();
    z
}

fn index_author(path: &str) -> Vec<Author> {
    let walkdir = WalkDir::new(path);
    let authors = walkdir.into_iter()
        .filter(|m| m.as_ref().unwrap().file_type().is_dir())
        .map(|f| f.unwrap())
        .collect::<Vec<_>>();

    let z = authors.iter()
    .enumerate()
    .map(|x| {
        
        Author::new(
            x.0,
            x.1.path().file_name().unwrap().to_str().unwrap().to_owned(),
            x.1.path().as_os_str().to_string_lossy().to_string(),
        ) 
    })
    .collect::<Vec<_>>();
    z
}


fn extract_manga_file_to_buffer(path : &str, buffer : &mut Vec<u8>, page: usize) {
    let test_file = File::open(path).unwrap();

    let file_buffer = BufReader::new(test_file);
    let mut zip = zip::ZipArchive::new(file_buffer);
    let mut buffered_zip = zip.unwrap();
    let manga_length = buffered_zip.len();

    let mut file_in_zip = buffered_zip.by_index(page).unwrap();
    
    
    file_in_zip.read_to_end( buffer);
}
