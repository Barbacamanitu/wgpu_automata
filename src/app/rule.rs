use bytemuck::{Pod, Zeroable};
use regex::Regex;

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Rule {
    pub born: [u32; 8],
    pub stay_alive: [u32; 8],
}

#[derive(Debug)]
pub enum RuleCreationError {
    InvalidRuleString,
}

impl Rule {
    pub fn from_rule_str(rstr: &str) -> Result<Rule, RuleCreationError> {
        let re_str = r#"B(\d+)/S(\d+)"#;
        let re: Regex = Regex::new(re_str).unwrap();
        let caps = re.captures(rstr);
        match caps {
            Some(c) => {
                if c.len() != 3 {
                    return Err(RuleCreationError::InvalidRuleString);
                }
                let born = c[1].chars();
                let stay = c[2].chars();
                let mut born_ints: [u32; 8] = [0; 8];
                let mut stay_ints: [u32; 8] = [0; 8];
                for b in born {
                    let b_int = b.to_digit(10);

                    match b_int {
                        Some(b_int_s) => {
                            if b_int_s == 0 {
                                return Err(RuleCreationError::InvalidRuleString);
                            }
                            born_ints[(b_int_s - 1) as usize] = 1;
                        }
                        None => return Err(RuleCreationError::InvalidRuleString),
                    }
                }

                for s in stay {
                    let s_int = s.to_digit(10);

                    match s_int {
                        Some(s_int_s) => {
                            if s_int_s == 0 {
                                return Err(RuleCreationError::InvalidRuleString);
                            } else {
                                stay_ints[(s_int_s - 1) as usize] = 1
                            }
                        }
                        None => return Err(RuleCreationError::InvalidRuleString),
                    }
                }
                Ok(Rule {
                    born: born_ints,
                    stay_alive: stay_ints,
                })
            }
            None => Err(RuleCreationError::InvalidRuleString),
        }
    }
}
