macro_rules! compiling_error {
    ($span: expr, $($arg: tt)*) => {
        syn::Error::new($span, format!($($arg)*))
            .to_compile_error()
            .into()
    };
}

pub(crate) use compiling_error;

/// Convert to camelCase:
///
/// * "HttpClient" => "httpClient"
/// * "httpClient" => "httpClient"
/// * "HTTPClient" => "httpClient"
pub(crate) fn to_camel_case(str: &str) -> String {
    let mut ret = String::new();
    let mut upper_state: Vec<bool> = vec![];
    for ch in &str.chars().collect::<Vec<_>>() {
        if ch.is_alphabetic() {
            upper_state.push(ch.is_uppercase());
        }
    }

    // * "HttpClient" => 1000 100000 => 0000 100000
    // * "httpClient" => 0000 100000 => 0000 100000
    // * "HTTPClient" => 1111 100000 => 0000 100000

    for (index, is_upper) in upper_state.clone().iter().enumerate() {
        if *is_upper && (index < upper_state.len() - 1) && upper_state[index + 1] {
            upper_state[index] = false;
        }
    }

    if upper_state[0] {
        upper_state[0] = false;
    }

    for (index, ch) in str.chars().collect::<Vec<_>>().iter().enumerate() {
        if upper_state[index] {
            ret.push(*ch);
        } else {
            ret.push(*ch.to_lowercase().collect::<Vec<_>>().first().unwrap());
        }
    }
    ret
}

/// Convert to camelCase:
///
/// * "HttpClient" => "HttpClient"
/// * "httpClient" => "HttpClient"
/// * "HTTPClient" => "HttpClient"
pub(crate) fn to_pascal_case(str: &str) -> String {
    let mut ret = String::new();
    let mut upper_state: Vec<bool> = vec![];
    for ch in &str.chars().collect::<Vec<_>>() {
        if ch.is_alphabetic() {
            upper_state.push(ch.is_uppercase());
        }
    }

    // * "HttpClient" => 1000 100000 => 0000 100000
    // * "httpClient" => 0000 100000 => 0000 100000
    // * "HTTPClient" => 1111 100000 => 0000 100000

    for (index, is_upper) in upper_state.clone().iter().enumerate() {
        if *is_upper && (index < upper_state.len() - 1) && upper_state[index + 1] {
            upper_state[index] = false;
        }
    }

    if upper_state[0] {
        upper_state[0] = true;
    }

    for (index, ch) in str.chars().collect::<Vec<_>>().iter().enumerate() {
        if upper_state[index] {
            ret.push(*ch);
        } else {
            ret.push(*ch.to_lowercase().collect::<Vec<_>>().first().unwrap());
        }
    }
    ret
}

/// Convert to snake_case:
///
/// * "HttpClient" => "http_client"
/// * "httpClient" => "http_client"
/// * "HTTPClient" => "http_client"
pub(crate) fn to_snake_case(str: &str) -> String {
    let mut ret = String::new();
    for (index, ch) in str.chars().collect::<Vec<_>>().iter().enumerate() {
        if index != 0 && ch.is_uppercase() {
            ret.push('_');
        }
        ret.push(*ch.to_lowercase().collect::<Vec<_>>().first().unwrap());
    }
    ret
}
