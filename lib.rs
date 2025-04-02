use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonData {
    Text(String),
    Data(HashMap<String, JsonData>),
    Arr(Vec<JsonData>),
    Num(f64),
    Bool(bool),
    Null,
    None
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

pub fn parse(json: &str) -> JsonData { // &impl AsRef<str>, Result<JsonData, String>
    let binding = json.to_string();
    let mut chars = binding.chars();
    parse_fragment(&mut chars).0
}

pub fn parse_fragment<I>(chars: &mut I ) -> (JsonData,char)
    where I: Iterator<Item = char>,  {
    let mut field_value = String::new();
     let mut field_name = String::new();
     let mut num_value = 0.0;
     let mut mant_dig = 1.0;
     let mut exp_val = 0.0;
     let mut neg = false;
     let mut neg_exp = false;
     let mut arr = Vec::new();
     let mut obj = HashMap::new();
    let mut state = Default::default();
    while let Some(c) = chars.next() {
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
                    _ => todo!("state {state:?}")
               }
           }
           ' ' | '\t' | '\r' | '\n' => {
               match state {
                   JsonState::Start | JsonState::ArrState => (),
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
                    _ => todo!("state {state:?}")
                   
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
                    _ => todo!("state {state:?}")
                   
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
                    _ => todo!("state {state:?}")
                   
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
                    _ => todo!()
                   
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
                   
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
                    _ => todo!("state {state:?}")
                    
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
                    _ => todo!("state {state:?} for {c}")
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
                    _ => todo!()
                   
                }
            }
            '-' => {
                match state {
                    JsonState::ObjName => field_name.push(c),
                    JsonState::ObjData => field_value.push(c),
                    JsonState::Start => state = JsonState::NegNum,
                    JsonState::ExpExpValue => state = JsonState::NegExpNum,
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    JsonState::Start => {
                        state = JsonState::BoolT
                    }
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?}")
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
                    JsonState::Start => {
                        state = JsonState::NulN
                    }
                    _ => todo!("state {state:?}")
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
                    _ => todo!("state {state:?} for {c}")
                   
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
}