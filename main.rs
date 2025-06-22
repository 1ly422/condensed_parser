#![allow(unused_parens)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//(A:B) = A -> B
// A,B = (A:B)
//(A:B,C) = (A:B), (A:C)
//<A:B> = (A :B), (B:A)
//<A:B,C> = (A:B), (B:A), (A:C), (C:A)

macro_rules! errorln {
    ($($arg:tt)*) => {
        println!("[ERROR]: {}", format!($($arg)*));
    };
}

#[derive(Debug)]
struct Pair {
    origin: String,
    destination: String,
}

struct TokenState {
    currentToken: usize,
    tokens: Vec<String>,
    result: Vec<Pair>,
}

impl TokenState {
    pub fn init() -> Self {
        return Self{currentToken: 0, tokens: Vec::new(), result: Vec::new()};
    }

    pub fn parse(self: &mut Self, input: &str) {
        self.lexer(input);
        println!("Lexed to {:?}", self.tokens);
        self.parse_tokens();
    }

    fn lexer(self: &mut Self, input: &str) {
        self.tokens.clear();
        let mut current = String::new();
        
        for ch in input.chars() {
            match ch {
                ',' | '<' | '>' | '(' | ')' | ':' => {
                    // if we have accumulated letters, push them first
                    if !current.is_empty() {
                        self.tokens.push(current.clone());
                        current.clear();
                    }
                    // push the symbol as a separate token
                    self.tokens.push(ch.to_string());
                }
                c if c.is_alphanumeric() => {
                    // accumulate letters and digits
                    current.push(c);
                }
                _ => {
                    // ignore any whitespace or unknown chars for now
                }
            }
        }
    
        // push last accumulated token if any
        if !current.is_empty() {
            self.tokens.push(current);
        }
    }

    fn parse_tokens(self: &mut Self) {
        let mut success = true;
        self.currentToken = 0;
        while (self.currentToken < self.tokens.len()) {
            let tokenType = to_token_type(&self.tokens[self.currentToken]);
            match tokenType {
                TokenType::OPEN_PAREN => {
                    println!("Paren parsing");
                    success = self.parse_parens_style();
                }
                TokenType::OPEN_ANGLE => {
                    println!("Angle parsing");
                    success = self.parse_plouk_style();
                }
                TokenType::CODE => {
                    println!("Legacy parsing");
                    success = self.parse_legacy_style();
                }
                TokenType::COMMA => {
                    //do nothing it should go to next token if available
                    success = true;
                }
                _ => {
                    errorln!("[state: {}]: Unknown starting token '{}'", &self.currentToken, &self.tokens[self.currentToken]);
                    success = false;
                }
            }
            if (!success) {
                break;
            }
            self.currentToken += 1;
        }
    
        if (!success) {
            println!("Parsing aborted")
        }
    }

    fn current(self : &Self) -> &String {
        return &self.tokens[self.currentToken];
    }
 
    fn add(self: &mut Self, origin: &String, destination: &String) {
        self.result.push(Pair{origin: origin.clone(), destination: destination.clone()});
    }

    fn expect(self: &mut Self, expectedtokenType: TokenType) -> Option<String> {
        self.currentToken += 1;
        if (self.currentToken < self.tokens.len()) {
            let tokenType = to_token_type(&self.tokens[self.currentToken]); 
            if (tokenType == expectedtokenType) {
                return Some(self.tokens[self.currentToken].clone())
            }
            else {
                let min = if (self.currentToken > 2) {self.currentToken - 2} else { 0 };
                let max = if (self.currentToken + 2 < self.tokens.len() - 1) {self.currentToken + 2} else { self.tokens.len() - 1 };
    
                errorln!("Expected {:?} but found {:?} at {}", expectedtokenType, tokenType, self.currentToken);
                return None;
            }
        }
        else {
            errorln!("An airline code should be followed by a {} but we reached the end already", ',',);
            return None;
        }
    }

    fn peak(self: &Self) -> TokenType {
        if (self.currentToken + 1 < self.tokens.len()) {
            return to_token_type(&self.tokens[self.currentToken + 1]);
        }
        return TokenType::UNKNOWN;
    }
    
    fn parse_legacy_style(self: &mut Self) -> bool {
        let origin = self.current().clone();
        let _ = self.expect(TokenType::COMMA);
        
        let destination = self.expect(TokenType::CODE);
        if let Some(d) = destination {
            self.add(&origin,  &d);
            return true;
        }
        else {
            eprintln!("Found an origin: {} but not destination", origin);
            eprintln!("In Legacy syntax an origin code must be followed by a destination as follow: RER,CDG")
        }
        return false;
    }
    
    fn parse_new_style(self: &mut Self, mode: ParenMode) -> bool {
        let CLOSING_TOKEN = mode.closing_token();
        let _ = self.current().clone();
    
        let origin = self.expect(TokenType::CODE);
        if (origin.is_none()) {
            return false;
        }
        
        let dotdot = self.expect(TokenType::DOTDOT);
        if (dotdot.is_none()) {
            return false;
        }
        
        let mut destinations: Vec<String> = Vec::new();
        let d = self.expect(TokenType::CODE);
        if (d.is_none()) {
            return false;
        }
        else {
            destinations.push(d.unwrap());
        }
        
        let mut nextToken = self.peak();
        while (nextToken != CLOSING_TOKEN) {
            match &nextToken {
                TokenType::COMMA => {
                    let comma = self.expect(TokenType::COMMA);
                    let d = self.expect(TokenType::CODE);
                    if (d.is_none()) {
                        errorln!("Found a comma but did not found a following code");
                        return false;
                    }
                    else {
                        destinations.push(d.unwrap());
                    }
                }
                _ => {
                    errorln!("Unexpected token found '{:?}' in code list", nextToken);
                    return false;
                }
            }
            //state.currentToken += 1;
            //println!("at {}", state.currentToken);
            nextToken = self.peak(); 
        }
    
        let closingParen = self.expect(CLOSING_TOKEN);
        if (closingParen.is_none()) {
            return false;
        }
        println!("destination: {:?}", destinations);
    
    
        match mode {
            ParenMode::ANGLE => {
                for d in destinations.iter() {
                    self.add(&origin.clone().unwrap(), d);
                    self.add(d, &origin.clone().unwrap());
                }
            }
    
            ParenMode::PAREN => {
                for d in destinations.iter() {
                    self.add(&origin.clone().unwrap(), d);
                }       
            }
        }
        
        return true;
    }
    
    fn parse_parens_style(self: &mut Self) -> bool {
        return self.parse_new_style(ParenMode::PAREN);
    }
    
    fn parse_plouk_style(self: &mut Self) -> bool {
        return self.parse_new_style(ParenMode::ANGLE);
    }
    
}

fn is_upper(t: &String) -> bool {
    for c in t.chars() {
        if (!c.is_uppercase()) {
            return false;
        }
    }
    return true;
}

fn is_alpha(t: &String) -> bool {
    for c in t.chars() {
        if (!c.is_ascii_alphabetic()) {
            return false;
        }
    }
    return true;
}

#[derive(PartialEq, Debug)]
enum TokenType {
    COMMA,
    CODE,
    DOTDOT,
    OPEN_PAREN,
    OPEN_ANGLE,


    CLOSING_PAREN,
    CLOSING_ANGLE,

    UNKNOWN,
}

#[derive(PartialEq, Debug)]
enum ParenMode {
    PAREN,
    ANGLE,
}

impl ParenMode {
    pub fn opening_token(self: &Self) -> TokenType {
        match self {
            ParenMode::ANGLE => {
                return TokenType::OPEN_ANGLE;
            }
            ParenMode::PAREN => {
                return TokenType::OPEN_PAREN;
            }
        }
    }

    pub fn closing_token(self: &Self) -> TokenType {
        match self {
            ParenMode::ANGLE => {
                return TokenType::CLOSING_ANGLE;
            }
            ParenMode::PAREN => {
                return TokenType::CLOSING_PAREN;
            }
        }
    }
}

fn to_token_type(t: &String) -> TokenType {
    if (t == ",") {
        return TokenType::COMMA;
    }
    else if (t == ":") {
        return TokenType::DOTDOT;
    }
    else if (t == "(") {
        return TokenType::OPEN_PAREN;
    }
    else if (t == "<") {
        return TokenType::OPEN_ANGLE;
    }
    else if (t == ")") {
        return TokenType::CLOSING_PAREN;
    }
    else if (t == ">") {
        return TokenType::CLOSING_ANGLE;
    }
    else if (is_alpha(t) && is_upper(t)) {
        return TokenType::CODE;
    }
    return TokenType::UNKNOWN;
}

fn main() {
    let inputNonValid =  "LHR,CDG,LON,(MAD:EUR),TRY"; // pair is separated by other syntax
    let inputNonValid2 = "LHR,CDG,LON,TRY,(MAD:EUR),JIL"; //without a pair at the end
    let inputNonValid3 = "LHR,CDG,<BTC,ADA>, LON,TRY,(MAD:EUR)";
    let inputNonValid4 = "LHR,CDG,<BTC,ADA>, LON,TRY,(MAD,EUR)";
    let inputNonValid5 = "LHR,CDG,(BTC,ADA), LON,TRY,(MAD,EUR)";
    let inputNonValid6 = "LHR,CDG,LON,(MAD:EUR),TRY,CDG,LHR,ORY,MAC";
    let inputNonValid7 = "LHR,CDG,LON,(MAD:EUR:TRL,TRY,CDG,LHR,ORY,MAC";
    let inputNonValid8 = "LHR,CDG,LON,<MAD,EUR:TRL>,TRY,CDG,LHR,ORY,MAC";
    let inputNonValid9 = "LHR,CDG,LON,(MAD:EUR:TRL),TRY,CDG,LHR,ORY,MAC";
    
    let simpleInputValid = "LHR,CDG,LON,TRY";
    let inputValid = "LHR,CDG,LON,TRY,(MAD:EUR)";
    let inputValid2 = "LHR,CDG,<BTC: ADA>, LON,TRY,(MAD:EUR)";
    let inputValid3 = "LHR,CDG,LON,TRY,(MAD:EUR),CDG,LHR,ORY,MAC";
    let inputValid4 = "LHR,CDG,LON,TRY,(MAD:EUR,JHG,NEM,NOM),CDG,LHR,ORY,MAC";
    let inputValid5 = "LHR,CDG,LON,TRY,<MAD:EUR,JHG,NEM,NOM>CDG,LHR,(BSD:BFF),ORY,MAC";
    
    let inputNonValid10 = "LHR,CDG(MAD:EUR)TRY,CDG"; //This is not valid but it still works fine with the implementation a BUG or a FEATURE???
    let inputNonValid11 = "LHR,CDG,LON,TRY,<MAD:EUR,JHG,NEM,NOM)>CDG,LHR,(BSD:BFF),ORY,MAC";
    //                                                         ^
    //                                               this should not be here
    let inputNonValid12 = "<MAD:LOL,NOM)>";

    let input = inputNonValid11;
    let mut state: TokenState = TokenState::init();
    println!("Input: \"{}\"", input);
    
    state.parse(input);

    println!("//////////////// Results ////////////////\n");

    for p in state.result.iter() {
        println!("{} -> {}", p.origin, p.destination);
    }
}
