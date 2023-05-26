use std::collections::HashMap;

use crate::makepad_micro_serde::{DeBin, SerBin};
use crate::makepad_live_id::*;

#[derive(PartialEq, Debug)]
pub struct HttpRequest {
    pub id: LiveId,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest { 
    // TODO: a good default
    pub fn new(url: String, method: String) -> Self {
        HttpRequest {
            id: LiveId::unique(),
            url,
            method,
            headers: HashMap::new(),
            body: None
        }
    }

    pub fn set_header(&mut self, name: String, value: String) {
        let entry = self.headers.entry(name).or_insert(Vec::new());
        entry.push(value);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = Some(body.into_bytes());
    }

    pub fn get_headers_string(&self) -> String {
        let mut headers_string = String::new();
        for (key, value) in self.headers.iter() {
            headers_string.push_str(&format!("{}: {}\n", key, value.join(",")));
        }
        headers_string
    }

    // WIP - takes whatever the user sends like a struct and we serialize to a byte array.
    // if it's possible I'd always send the body as a byte array to java to avoid 
    // sending a generic body and doing parsing/serializing on that side.
    // if we can't rely to always send byte array in the body,
    // // we could use the header's content-type and use that to know what to serialize into.
    // pub fn set_body<T: DeBin + SerBin + std::fmt::Debug>(&mut self, body: T) {
    //     self.body = Some(body.serialize_bin()); 
    // }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub id: LiveId,
    pub status_code: u16,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new(id: LiveId, status_code: u16, body: Option<Vec<u8>>) -> Self {
        HttpResponse {
            id,
            status_code,
            headers: HashMap::new(),
            body: body
        }
    }
    
    // For now it's only String, let's see how it develops
    pub fn get_body(&self) -> Option<String> { 
        if let Some(body) = self.body.as_ref() {
            let deserialized = String::from_utf8(body.to_vec()).unwrap();
            Some(deserialized)
        } else {
            None
        }
    }

    // TODO avoid duplication of this function
    pub fn get_headers_string(&self) -> String {
        let mut headers_string = String::new();
        for (key, value) in self.headers.iter() {
            headers_string.push_str(&format!("{}: {}\r\n", key, value.join(",")));
        }
        headers_string
    }

    pub fn parse_and_set_headers(&mut self, headers_string: String) {
        let mut headers = HashMap::new();
        for line in headers_string.lines() {
            let mut split = line.split(":");
            let key = split.next().unwrap();
            let values = split.next().unwrap().to_string();
            for val in values.split(",") {
                let entry = headers.entry(key.to_string()).or_insert(Vec::new());
                entry.push(val.to_string());
            }
        }
        self.headers = headers;
    }

    // I'm almost sure this is not going to work, since the serialization format
    // is very specific to Makepad.
    // pub fn get_body<T: DeBin + SerBin>(&self) -> Option<T> { 
    //     if let Some(body) = self.body.as_ref() {
    //         crate::log!("body: {:?}", body);
    //         let deserialized: T = DeBin::deserialize_bin(&body).unwrap(); //TODO: return result
    //         Some(deserialized)
    //     } else {
    //         None
    //     }
    // }
}