
#[macro_export]
macro_rules! warning {
    ($name:expr) => {
        println!("[ {} ] | [ {} ]", ansi_term::Color::Red.bold().paint("[ ! ]"), $name);
    };
    ($name:expr, $greppable:expr, $accessible:expr) => {
        if !$greppable {
            if $accessible {
                println!("[ {} ]", $name);
            } else {
                println!("[ {} ] | [ {} ]", ansi_term::Color::Red.bold().paint("[ ! ]"), $name);
            }
        }
    };
}



#[macro_export]
macro_rules! detail {
    ($name:expr) => {
        println!("[ {} ] | [ {} ]", ansi_term::Color::Blue.bold().paint("[ ~ ]"), $name);
    };

    ($name:expr, $greppable:expr, $accessible:expr) => {
        if !$greppable {
            if $accessible {
                println!("[ {} ]", $name);
            } else {
                println!("[ {} ] | [ {} ]", ansi_term::Color::Blue.bold().paint("[ ~ ]"), $name);
            }
        }
    };
}



#[macro_export]

macro_rules! output {
    ($name:expr) => {
        println!("[ {} ] | [ {} ]", RGansi_term::Color::RGB(0, 255, 9).bold().paint("[ > ]"), $name);
    };

    ($name:expr, $greppable:expr, $accessible:expr) => {
        if !$greppable {
            if $accessible {
                println!("[ {} ]", $name);
            } else {
                println!("[ {} ] | [ {} ]", RGansi_term::Color::RGB(0, 255, 9).bold().paint("[ > ]"), $name);
            }
        }
    };

}



#[macro_export]
macro_rules! opening {
    () => {
        use rand::seq::SliceRandom;
        let quotes = vec![
            // TODO():
        ];
        let random_quote = quotes.choose(&mut rand::thread_rng()).unwrap();
        println!("[ {} ]", random_quote);
    };
}












