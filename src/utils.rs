use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/');

pub fn url_encode(component: String) -> String {
  utf8_percent_encode(&component, FRAGMENT).to_string()
}
