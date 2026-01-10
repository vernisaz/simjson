use std::{collections::HashMap,char};

pub const VERSION: &str = env!("VERSION");

#[derive(Debug, Clone, PartialEq)]
pub enum JsonData {
    Text(String),
    Data(HashMap<String, JsonData>),
    Arr(Vec<JsonData>),
    Num(f64),
    Bool(bool),
    Null,
    None,
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
enum JsonState {
#[default]
    Start,
    ObjState,
    ObjName,
    ObjData,
    NumValue,
    MantissaValue,
    ExpNumValue,
    ExpNameSep,
    ExpExpValue,
    ObjExpEnd,
    NegExpNum,
    ArrState,
    NegNum,
    ArrNext,
    EscValue,
    EscName,
    UniDigVal,
    UniDigName,
    ErrState,
    BoolT,
    BoolR,
    BoolU,
    BoolF,
    BoolA,
    BoolL,
    BoolS,
    NulN,
    NulU,
    NulL
}

#[macro_export]
macro_rules! error {
    () => {
        return (JsonData::Err("an error happened".to_string()), 0 as char)
    };
    ($($arg:tt)+) => {
        return (JsonData::Err(format!("an error: {}", format_args!($($arg)+))), 0 as char)
    };
}

pub fn get_path_as_text(json: &JsonData, path: &impl AsRef<str>) -> Option<String> {
    let comps = path.as_ref().split('/');
    let mut json = json;
    for cur in comps {
        if let JsonData::Data(json_n) = json {
            match json_n.get(cur) {
                 Some(json_n) => json = json_n,
                 _ => return None
            }
        }
    }
    match json {
        JsonData::Text(text) => Some(text.clone()),
        _ => None
    }
}

pub fn parse(json: &str) -> JsonData { // &impl AsRef<str>, Result<JsonData, String>
    let binding = json.to_string();
    let mut chars = binding.chars();
    parse_fragment(&mut chars).0
}

pub struct JsonStr <'a> {
    chars: &'a mut dyn Iterator<Item=char>
}

impl Iterator for JsonStr<'_>  {
    type Item = JsonData;
    
    fn next(&mut self) -> Option<Self::Item> {
    
        let res = parse_fragment(self.chars);
        match res.0 {
            JsonData::None => None,
            data => Some(data)
        }
    }
}

pub fn parse_fragment<I>(chars: &mut I ) -> (JsonData,char) 
    where I: Iterator<Item = char> + ?Sized,  {
    let mut field_value = String::new();
     let mut field_name = String::new();
     let mut num_value = 0.0;
     let mut mant_dig = 1.0;
     let mut exp_val = 0.0;
     let mut neg = false;
     let mut neg_exp = false;
     let mut arr = Vec::new();
     let mut obj = HashMap::new();
     let mut dig_inx = String::new(); dig_inx.reserve(4);
     let mut surrogates = [0_u16,0]; let mut surrogate_first = true;
    let mut state = Default::default();
    let mut char_pos = 0;
    let mut char_line = 1;
    while let Some(c) = chars.next() {
        if c == '\n' {
            char_pos = 0;
            char_line += 1;
        }
        match c {
           '"' => {
               match state {
                    JsonState::Start => {
                        field_value .clear();
                        state = JsonState::ObjData},
                    JsonState::ObjState => {
                        state = JsonState::ObjName;
                        field_name .clear();
                    },
                    JsonState::ObjData => {
                        return (JsonData::Text(field_value.clone()),c)
                    },
                    JsonState::ObjName => {
                        state = JsonState::ExpNameSep;
                    },
                    JsonState::EscName => {
                        field_name.push(c);
                        state = JsonState::ObjName
                    },
                    JsonState::EscValue => {
                        field_value.push(c);
                        state = JsonState::ObjData
                    },
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
               }
           }
           ' ' | '\t' | '\r' | '\n' => {
               match state {
                   JsonState::Start | JsonState::ArrState | JsonState::ArrNext => (),
                   JsonState::ObjName => {
                        field_name.push(c)
                   }
                   JsonState::ObjData => {
                        field_value.push(c)
                   },
                   JsonState::ObjState | JsonState::ObjExpEnd => (),
                   JsonState::NumValue => {
                        if neg { num_value = -num_value}
                        return (JsonData::Num(num_value),c)
                   }
                   JsonState::ExpNameSep => {
                   }
                   JsonState::EscValue => {
                        state = JsonState::ObjData;
                        //field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
               }
            }
            '[' => {
                match state {
                    JsonState::Start => {
                        arr.clear();
                        let fragment = parse_fragment(chars);
                        arr.push(fragment.0);
                        match fragment.1 {
                            ',' => {
                                loop {
                                    let fragment = parse_fragment(chars);
                                    arr.push(fragment.0);
                                    match fragment.1 {
                                        ']' => return (JsonData::Arr(arr.clone()),c),
                                        ',' => continue,
                                        _ => break
                                    }
                                }
                                state = JsonState::ArrNext
                            },
                            ']' => return (JsonData::Arr(arr.clone()),c),
                            _ => state = JsonState::ArrState
                        }
                    }
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    },
                    JsonState::ArrState => {
                        arr.push(parse_fragment(chars).0)
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
                }
            }
            '{' => {
                match state {
                    JsonState::Start => {
                        state = JsonState::ObjState;
                        obj.clear();
                    }
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    },
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            '\\' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        state = JsonState::EscName
                    }
                    JsonState::ObjData => {
                        state = JsonState::EscValue
                    },
                    JsonState::EscName => {
                        field_name.push(c);
                        state = JsonState::ObjName
                    },
                    JsonState::EscValue => {
                        field_value.push(c);
                        state = JsonState::ObjData
                    },
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
                }
            }
            '/' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::EscName => {
                        field_name.push(c);
                        state = JsonState::ObjName
                    },
                    JsonState::EscValue => {
                        field_value.push(c);
                        state = JsonState::ObjData
                    },
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
                }
            }
            ':' => {
                match state {
                    JsonState::Start  => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::ExpNameSep => {
                        let fragment = parse_fragment(chars);
                        obj.insert(field_name.clone(),  fragment.0);
                       // chars.next_back();
                        match fragment.1 {
                            '}' => {
                                return (JsonData::Data(obj.clone()),c)
                            }
                            ',' => state = JsonState::ObjState,
                            _ => state = JsonState::ObjExpEnd
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            ']' => {
                match state {
                    JsonState::Start => state = JsonState::ArrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    },
                    JsonState::MantissaValue | JsonState::NumValue => {
                        if neg {num_value = -num_value}
                        return (JsonData::Num(num_value),c)
                       // arr.push(JsonData::Num(num_value));
                       // return (JsonData::Arr(arr.clone()),c)
                    }
                    JsonState::ExpNumValue => {
                        if neg {num_value =- num_value}
                        if neg_exp {exp_val = -exp_val}
                        return (JsonData::Num(num_value *  10.0_f64.powf(exp_val)),c)
                    }
                    JsonState::ArrState | JsonState::ArrNext => return (JsonData::Arr(arr.clone()),c),
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
                }
            }
            '}' => {
                match state {
                    JsonState::Start => return (JsonData::None,c),
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::NumValue |  JsonState::MantissaValue => {
                        if neg {num_value =- num_value}
                        return (JsonData::Num(num_value),c)
                    }
                    JsonState::ExpNumValue => {
                        if neg {num_value =- num_value}
                        if neg_exp {exp_val = -exp_val}
                        return (JsonData::Num(num_value *  10.0_f64.powf(exp_val)),c)
                    }
                    JsonState::ObjExpEnd => return (JsonData::Data(obj.clone()),char::from_u32(0).unwrap()),
                    JsonState::ObjState => return (JsonData::None,c),
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                    
                }
            }
            '0' ..= '9' => {
                match state {
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::NumValue => {
                        num_value = num_value * 10.0 + c.to_digit(10).unwrap() as f64
                    },
                    JsonState::MantissaValue => {
                        mant_dig *= 10.0;
                        num_value += (c.to_digit(10).unwrap() as f64) / mant_dig;
                    }
                    JsonState::ExpNumValue => {
                       // state = JsonState::NumValue;
                        exp_val = exp_val * 10.0 + c.to_digit(10).unwrap() as f64
                    }
                    JsonState::NegExpNum => {
                        state = JsonState::ExpNumValue;
                        exp_val = c.to_digit(10).unwrap() as _;
                        neg_exp = true;
                    }
                    JsonState::ExpExpValue => {
                        state = JsonState::ExpNumValue;
                        neg_exp = false;
                        exp_val = c.to_digit(10).unwrap() as _
                    }
                    JsonState::Start => {
                        state = JsonState::NumValue;
                        mant_dig = 1.0;
                        exp_val = 0.0;
                        neg = false;
                        num_value = c.to_digit(10).unwrap() as _
                    }
                    JsonState::NegNum => {
                        state = JsonState::NumValue;
                        mant_dig = 1.0;
                        exp_val = 0.0;
                        num_value = c.to_digit(10).unwrap() as _;
                        neg = true;
                    }
                    JsonState::ArrState => {
                        state = JsonState::NumValue; 
                        mant_dig = 1.0;
                         exp_val = 0.0;
                        neg = false;
                        num_value = c.to_digit(10).unwrap() as _
                    }
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) { // as u32
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                            if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                            match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            '.' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::NumValue => {
                        field_value.push(c);
                        state = JsonState::MantissaValue
                    },
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!()
                   
                }
            }
            '-' => {
                match state {
                    JsonState::ObjName => field_name.push(c),
                    JsonState::ObjData => field_value.push(c),
                    JsonState::Start => state = JsonState::NegNum,
                    JsonState::ExpExpValue => state = JsonState::NegExpNum,
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'E' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::MantissaValue | JsonState::NumValue => {
                        //exp_val = 0.0
                        state = JsonState::ExpExpValue
                    },
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) {
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                            if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                             match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            ',' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    /*JsonState::ArrNumValue => {
                        arr.push(JsonData::Num(if neg {-num_value} else {num_val}));
                        state = JsonState::ArrNext
                    }*/
                    JsonState::NumValue | JsonState::MantissaValue => {
                        return (JsonData::Num(if neg {-num_value} else {num_value}),c)
                    }
                    JsonState::ExpNumValue => {
                        if neg {num_value = - num_value};
                        if neg_exp {exp_val = -exp_val}
                        return (JsonData::Num(num_value * 10.0_f64.powf(exp_val)),c)
                    }
                    JsonState::ObjExpEnd => {
                        state = JsonState::ObjState},
                    JsonState::ArrState  => {
                        arr.push(parse_fragment(chars).0);
                        state = JsonState::ArrNext
                    }
                    JsonState::ArrNext => {
                        let fragment = parse_fragment(chars);
                        arr.push(fragment.0);
                        match fragment.1 {
                            ',' => {
                                loop {
                                    let fragment = parse_fragment(chars);
                                    arr.push(fragment.0);
                                    match fragment.1 {
                                        ']' => return (JsonData::Arr(arr.clone()),c),
                                        ',' => continue,
                                        _ => break
                                    }
                                }
                                state = JsonState::ArrNext
                            },
                            ']' => return (JsonData::Arr(arr.clone()),c),
                            _ => state = JsonState::ArrState
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            't' => {
                 match state {
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\t')
                    }
                    JsonState::Start => {
                        state = JsonState::BoolT
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'r' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::BoolT => {
                        state = JsonState::BoolR
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\r')
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'u' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::BoolR => {
                        state = JsonState::BoolU
                    }
                    JsonState::NulN => {
                        state = JsonState::NulU
                    }
                    
                    JsonState::EscName => {
                        dig_inx.clear();
                        state = JsonState::UniDigName
                    },
                    JsonState::EscValue => {
                        dig_inx.clear();
                        state = JsonState::UniDigVal
                    },
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'U' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    
                    JsonState::EscName => {
                        dig_inx.clear();
                        state = JsonState::UniDigName
                    },
                    JsonState::EscValue => {
                        dig_inx.clear();
                        state = JsonState::UniDigVal
                    },
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'e' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::MantissaValue | JsonState::NumValue => {
                        //exp_val = 0.0
                        state = JsonState::ExpExpValue
                    },
                    JsonState::BoolU => {
                        return (JsonData::Bool(true),c)
                    }
                    JsonState::BoolS => {
                        return (JsonData::Bool(false),c)
                    }
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) {
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                            if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                             match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'f' => {
               match state {
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::Start => {
                        state = JsonState::BoolF
                    }
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) {
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                             if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                             match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push(12 as char);
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                } 
            }
            'a' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::BoolF => {
                        state = JsonState::BoolA
                    }
                    
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) {
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                            if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                             match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'l' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::BoolA => {
                        state = JsonState::BoolL
                    }
                    JsonState::NulU => {
                        state = JsonState::NulL
                    }
                    JsonState::NulL => {
                        return (JsonData::Null,c)
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            's' => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::BoolL => {
                        state = JsonState::BoolS
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                }
            }
            'n' => {
               match state {
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\n')
                    }
                    JsonState::Start => {
                        state = JsonState::NulN
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                } 
            }
            'b' | 'c' | 'd' | 'B' | 'C' | 'D' | 'A' | 'F' => {
                match state {
                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    }
                    JsonState::UniDigVal | JsonState::UniDigName => {
                        dig_inx.push(c) ;
                        if dig_inx.len() == 4 {
                            match u32::from_str_radix(&dig_inx, 16) {
                                Ok(number) => {//println!("The number is: {}", number);
                                    match std::char::from_u32(number) {
                                        Some(character) => {//println!("The character is: {}", character);
                                            match state {
                                                JsonState::UniDigVal => {
                                                    field_value.push(character);
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    field_name.push(character);
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                        None => {//eprintln!("Error: Invalid Unicode scalar value!");
                                            if !surrogate_first {
                                                surrogates[1] = number as u16;
                                                match char::decode_utf16(surrogates).next() {
                                                    Some(Ok(character)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(character)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(character)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    Some(Err(_)) => match state {
                                                        JsonState::UniDigVal => {
                                                            field_value.push(char::REPLACEMENT_CHARACTER)
                                                        }
                                                        JsonState::UniDigName => {
                                                             field_name.push(char::REPLACEMENT_CHARACTER)
                                                        },
                                                         _ => unreachable!()
                                                    }
                                                    None => unreachable!()
                                                }
                                            } else {
                                                surrogates[0] = number as u16;
                                            }
                                            surrogate_first = !surrogate_first;
                                             match state {
                                                JsonState::UniDigVal => {
                                                    state = JsonState::ObjData
                                                }
                                                JsonState::UniDigName => {
                                                    state = JsonState::ObjName
                                                },
                                                 _ => unreachable!()
                                            }
                                        },
                                    }
                                },
                                Err(e) => { eprintln!("Failed to convert: {}", e);
                                     match state {
                                        JsonState::UniDigVal => {
                                            state = JsonState::ObjData
                                        }
                                        JsonState::UniDigName => {
                                            state = JsonState::ObjName
                                        },
                                        _ => unreachable!()
                                    }
                                },
                            }
                        }
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        if c == 'b' {
                            field_value.push(0x8 as char);
                        } else {
                            field_value.push('\\');
                            field_value.push(c)
                        }
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                } 
            }
            _ => {
                match state {
                    JsonState::Start => state = JsonState::ErrState,

                    JsonState::ObjName => {
                        field_name.push(c)
                    }
                    JsonState::ObjData => {
                        field_value.push(c)
                    },
                    JsonState::EscName => {
                        state = JsonState::ObjName;
                        field_name.push('\\');
                        field_name.push(c)
                    }
                    JsonState::EscValue => {
                        state = JsonState::ObjData;
                        field_value.push('\\');
                        field_value.push(c)
                    }
                    _ => error!("state {state:?} for {c} at {char_pos}:{char_line}")
                   
                }
            }
        }
    }
    (JsonData::None,char::from_u32(0).unwrap())
}

pub fn esc_quotes(jstr: String) -> String {
    let mut res = String::new();
    for c in jstr.chars() {
        match c {
            '"' | '\\' => res.push('\\') ,
            _ => ()
        }
        res.push(c)
    }
    res
}

#[cfg(test)]
use JsonData::{Data,Arr};
#[cfg(test)]
fn main() {
    let res = parse("[{\"name\":\"malina\", \"age\":19},{}, 45.8]");
    println!{"{res:?}"}
    let res = parse("{\"name\":\"calina\", \"age\":39, \"husband\":{\"name\":\"Josef\", \"age\":65}, \"mid\":\"A\", \"kids\":[\"jef\", \"ruth\"], \"port\":400}");
    println!{"{res:?}"}
    let res = parse("[300,-42.6,1.562e45, 0.56e3]");
    println!{"{res:?}"}
    let res = parse(r#"[0.56e-2,5,32,54.08,-5.6,null,false,true,70e12,1.2E03]"#);
     println!{"{res:?}"}
    let res = parse(r#"[[0,5],[3,0.2],[{"a\"":"70" ,"b":"28", "S":true},{"c":"d\"","Mar":false,"x":[4, 8 ] }]]"#);
     println!{"{res:?}"}
    let json_str = r#"{"simple":"json"}
         {"another":true} ["again","stop"]
         {"not again":false}
"#;
    let mut chars = json_str.chars();
    loop {
        let res = parse_fragment(&mut chars);
        let json = match res.0 {
            Data(json) => println!("{json:?}"),
            Arr(json) => println!("{json:?}"),
            JsonData::None => {println!("end of fragments");break},
            _ => {eprintln!("invalid json {:?}", res.0);break},
        };
    }
}