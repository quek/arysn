use arysn_macro::defar;

defar!(User);

fn main() {
    let user = User {
        name: "にゃ～".to_string(),
    };
    println!("user {:?}", user);
}

#[cfg(test)]
mod tests {
    use arysn_macro::defar;

    defar!(Neko);

    #[test]
    fn it_works() {
        let neko = Neko {
            name: "こねら".to_string(),
        };
        assert_eq!("こねら", &neko.name);
    }
}
