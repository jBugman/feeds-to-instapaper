extern crate failure;
extern crate feeds_to_instapaper;

fn main() {
    if let Err(err) = feeds_to_instapaper::run() {
        let mut causes = err.causes();
        eprintln!("error: {}", causes.next().unwrap());
        for c in causes {
            eprintln!(" caused by: {}", c);
        }
        std::process::exit(1);
    };
}
