use std::{io::{self, Write}, fs::{self, OpenOptions, File}};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

const PATH: &str = "./database.json";

#[derive(Debug)]
enum Action {
    Login,
    CreateAccount,
    Exit,
    Unknown,
}
#[derive(Debug)]
enum UserAction {
    Deposit,
    Withdraw,
    Logout,
    Unknown
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    name: String,
    password: String,
    account: Account,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Account {
    account_number: Uuid,
    balance: i32,
}

type Users = Vec<User>;

impl From<&str> for UserAction {
    fn from(action: &str) -> Self {
        match action {
            "1" => Self::Deposit,
            "2" => Self::Withdraw,
            "3" => Self::Logout,
            _   => Self::Unknown
        }
    }
}

impl From<&str> for Action {
    fn from(action: &str) -> Self {
        match action {
            "1" => Self::Login,
            "2" => Self::CreateAccount,
            "3" => Self::Exit,
            _   => Self::Unknown,
        }
    }
}

impl Account {
    fn new() -> Self {
        Account { account_number: Uuid::new_v4(), balance: 0 }
    }

    fn deposit(&mut self, value: i32) -> Result<(), &'static str> {
        match value < 1 {
            true => Err("Can't deposit value smaller than 1"),
            false => {
                self.balance += value * 100;
                Ok(())
            }
        }
    }

    fn withdraw(&mut self, value: i32) -> Result<(), &'static str> {
        match value > self.balance / 100 {
            true => Err("Can't withdraw a value larger than current balance"),
            false => {
                self.balance -= value * 100;
                Ok(())
            }
        }
    }
}

impl PartialEq<User> for User {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.password == other.password
    }
}

impl User {
    fn new(name: &str, password: &str) -> Self {
        User { name: String::from(name), password: String::from(password), account: Account::new() }
    }

    fn deposit(&mut self, value: i32) -> Result<(), &'static str> {
        match self.account.deposit(value) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    fn withdraw(&mut self, value: i32) -> Result<(), &'static str> {
        match self.account.withdraw(value) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }


    fn login(username: &str, password: &str) -> Result<User, &'static str> {
        let user_to_find = User::new(username, password);

        let users = match get_users() {
            Ok(users) => users,
            Err(_) => {
                return Err("Error fetching users");
            }
        };

        match users.iter().find(|&user| *user == user_to_find) {
            Some(found_user) => {
                return Ok(found_user.to_owned());
            },
            None => {
                return Err("user not found");
            }
        };
    }
}

fn print_menu(menu_items: Vec<&str>) {
    println!("============================================");
    for (idx, &item) in menu_items.iter().enumerate() {
        let mut formatted_item = format!("| {}: {}", idx + 1, item);
        for _ in 1..44 - formatted_item.len() {
            formatted_item.push_str(" ")
        }
        formatted_item.push_str("|");
        println!("{}", formatted_item)
    }
    println!("============================================");
}

fn sign_up() -> Result<User, &'static str> {
     match get_db_file() {
        Ok(file) => {
            let mut users = match get_users() {
                Ok(users) => users,
                Err(error) => match error.is_eof() {
                    true => Vec::<User>::new(),
                    false => panic!("Something went wrong.")
                }
            };

            let name = read_input("name: ");

            let password = read_input("password: ");
            
            let user = User::new(&name.trim(), &password.trim());
            
            users.push(user.clone());
            update_users(file, users);
            return Ok(user);
        },
        Err(_) => {
            return Err("Error while openning file")
        },
    };
}

fn login() -> Result<User, &'static str> {
    User::login(read_input("Username: ").trim(), read_input("Password: ").trim())
}

fn get_users() -> Result<Users, serde_json::Error> {
    match serde_json::from_str::<Users>(fs::read_to_string(PATH).unwrap().to_string().as_str()) {
        Ok(users) => Ok(users),
        Err(error) => Err(error)
    }
}

fn update_users(mut file: File,users: Users) {
    file.write_all(serde_json::to_string_pretty(&users).unwrap().as_bytes()).expect("Error writing to file.")
}

fn read_input(message: &str) -> String {
    let mut input = String::new();
    print!("{}", message);
    io::Write::flush(&mut io::stdout()).expect("Flush failed.");
    io::stdin().read_line(&mut input).expect("Error reading input.");

    return input;
}

fn get_db_file() -> Result<File, String> {
    match OpenOptions::new().write(true).create(true).read(true).open(PATH) {
        Ok(file) => Ok(file),
        Err(error) => Err(error.to_string())
    }

}

fn deposit(user: &mut User) -> Result<(), &str> {
    let deposit_value = read_input("Value to deposit: ");
    match user.deposit(deposit_value.trim().parse::<i32>().unwrap()) {
        Ok(_) => {
            let users = match get_users() {
                Ok(users) => users,
                Err(error) => panic!("something went wrong: {:?}", error.to_string())
            };

            match users.iter().position(|curr_user| *user == *curr_user) {
                Some(idx) => {
                    let mut users = get_users().expect("Error getting users");
                    users[idx] = user.to_owned();
                    match get_db_file() {
                        Ok(file) => {
                            update_users(file, users)
                        }
                        Err(error) => panic!("Error opening the file: {:?}", error.to_string())
                    }
                    
                },
                None => panic!("User not found")
            }
            Ok(())
        },
        _ => Err("Something went wrong while depositing")
    }
}

fn withdraw(user: &mut User) -> Result<(), &str> {
    let withdraw_value = read_input("Value to withdraw: ");
    match user.withdraw(withdraw_value.trim().parse::<i32>().unwrap()) {
        Ok(_) => {
            let users = match get_users() {
                Ok(users) => users,
                Err(error) => panic!("something went wrong: {:?}", error.to_string())
            };

            match users.iter().position(|curr_user| *user == *curr_user) {
                Some(idx) => {
                    let mut users = get_users().expect("Error getting users");
                    users[idx] = user.to_owned();
                    match get_db_file() {
                        Ok(file) => {
                            update_users(file, users)
                        }
                        Err(error) => panic!("Error opening the file: {:?}", error.to_string())
                    }
                    
                },
                None => panic!("User not found")
            }
            Ok(())
        },
        _ => Err("Something went wrong while depositing")
    }
}

fn main() {
    print_menu(vec!["Login", "Sign up", "Exit"]);
    loop {
        let input = read_input("Choose an action: ");
        let result = match Action::from(input.trim()) {
            Action::Login => login(),
            Action::CreateAccount => sign_up(),
            Action::Exit => Err("Exit"),
            Action::Unknown => Err("Unknown"),
        };

        match result {
            Ok(mut user) => {
                print_menu(vec!["Deposit","Withdraw", "Logout"]);
                let user = &mut user;

                loop {
                    let input = read_input("Choose an action: ");

                    let result = match UserAction::from(input.trim()) {
                        UserAction::Deposit => deposit(user),
                        UserAction::Withdraw => withdraw(user),
                        UserAction::Logout => Err("Logout"),
                        UserAction::Unknown => Err("Unknown")
                    };

                    match result {
                        Ok(()) => continue,
                        Err("Unknown") => continue,
                        _ => break
                    }
                }
            },
            Err("Unknown") => continue,
            Err(_) => break
        }
    }
}
