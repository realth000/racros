#[derive(Debug, Clone)]
struct CharState {
    /// Current char is alphabec.
    alphabetic: bool,
    /// Current char is uppercase.
    uppercase: bool,
    /// Current char content.
    content: char,
}

macro_rules! compiling_error {
    ($span: expr, $($arg: tt)*) => {
        syn::Error::new($span, format!($($arg)*))
            .to_compile_error()
            .into()
    };
}

pub(crate) use compiling_error;

fn prepare_char_state(str: &str) -> Vec<CharState> {
    let mut state_list: Vec<CharState> = vec![];
    for ch in &str.chars().collect::<Vec<_>>() {
        if ch.is_alphabetic() {
            state_list.push(CharState {
                alphabetic: true,
                uppercase: ch.is_uppercase(),
                content: *ch,
            });
        } else {
            state_list.push(CharState {
                alphabetic: false,
                uppercase: false,
                content: *ch,
            });
        }
    }
    state_list
}

fn char_convert_uppercase(ch: &char) -> char {
    *ch.to_uppercase().collect::<Vec<_>>().first().unwrap()
}

fn char_convert_lowercase(ch: &char) -> char {
    *ch.to_lowercase().collect::<Vec<_>>().first().unwrap()
}

/// Internal combined method to implement conversion towards camel case and pascal case.
fn case_internal(str: &str, first_index_upper: bool) -> String {
    let mut ret = String::new();
    let state_list = prepare_char_state(str);

    // Flag to indicate continously indexing uppercase chars.
    let mut still_in_uppercase = false;
    let mut need_uppercase = false;

    for (index, state) in state_list.iter().enumerate() {
        if index == 0 {
            if state.uppercase {
                still_in_uppercase = true;
            }
            if first_index_upper {
                ret.push(char_convert_uppercase(&state.content));
            } else {
                ret.push(char_convert_lowercase(&state.content));
            }
        } else if still_in_uppercase {
            // Former letter is uppercase.
            if !state.uppercase {
                // Current letter is not uppercase.
                still_in_uppercase = false;
            }
            // Next letter exists and is not uppercase:
            // e.g. HTTPClient
            //          |
            //          current letter.
            // will convert into "HttpClient", keeps current letter uppercase if it is.
            //
            // e.g. HTTP_Client
            //         |
            //         current letter.
            // will convert into "HttpClient", next letter is not alphlabet, keep lowercase.
            if index < state_list.len() - 1
                && state_list[index + 1].alphabetic
                && !state_list[index + 1].uppercase
            {
                ret.push(state.content);
            } else if !state.alphabetic {
                still_in_uppercase = false;
                continue;
            } else {
                // Next letter exists and is not uppercase:
                // e.g. HTTPClient
                //         |
                //         current letter.
                // will convert into "HttpClient", current letter must be lowercase.
                ret.push(char_convert_lowercase(&state.content));
            }
        } else if need_uppercase {
            ret.push(char_convert_uppercase(&state.content));
            need_uppercase = false;
        } else if state.uppercase {
            ret.push(state.content);
            still_in_uppercase = true;
        } else if !state.alphabetic {
            need_uppercase = true;
        } else {
            ret.push(state.content);
            still_in_uppercase = false;
        }
    }
    ret
}

/// Convert to camelCase:
///
/// * `HttpClient` => `httpClient`
/// * `httpClient` => `httpClient`
/// * `HTTPClient` => `httpClient`
/// * `HTTP_CLIENT` => `httpClient`
pub fn to_camel_case(str: &str) -> String {
    case_internal(str, false)
}

/// Convert to camelCase:
///
/// * `HttpClient` => `HttpClient`
/// * `httpClient` => `HttpClient`
/// * `HTTPClient` => `HttpClient`
pub fn to_pascal_case(str: &str) -> String {
    case_internal(str, true)
}

/// Convert to snake case:
///
/// * `HttpClient` => `http_client`
/// * `httpClient` => `http_client`
/// * `HTTPClient` => `http_client`
pub fn to_snake_case(str: &str) -> String {
    let mut ret = String::new();
    let s = case_internal(str, false);
    for ss in s.chars() {
        if ss.is_uppercase() {
            ret.push('_');
            ret.push(char_convert_lowercase(&ss));
        } else {
            ret.push(ss);
        }
    }
    ret
}

/// Convert to screaming case:
///
/// * `HttpClient` => `HTTP_CLIENT`
/// * `httpClient` => `HTTP_CLIENT`
/// * `HTTPClient` => `HTTP_CLIENT`
pub fn to_screaming_case(str: &str) -> String {
    let mut ret = String::new();
    let s = case_internal(str, false);
    for ss in s.chars() {
        if ss.is_uppercase() {
            ret.push('_');
        } else {
            ret.push(char_convert_uppercase(&ss));
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRS1: [&str; 4] = ["httpClient", "HttpClient", "HTTPClient", "HTTP_CLIENT"];

    #[test]
    fn test_to_camel_case() {
        for s in STRS1 {
            assert_eq!(to_camel_case(s), "httpClient");
        }
    }

    #[test]
    fn test_to_pascal_case() {
        for s in STRS1 {
            assert_eq!(to_pascal_case(s), "HttpClient");
        }
    }

    #[test]
    fn test_snake_case() {
        for s in STRS1 {
            println!("{s}");
            assert_eq!(to_snake_case(s), "http_client");
        }
    }
}
