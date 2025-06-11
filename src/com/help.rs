pub fn main() {
    let str = r"
           __    Hello i'am Rex
          / _) /
   .-^^^-/ /
  /       /
<_.|_|-|_|

here are some commands you can use:

help                        - show this help message
ssl <command> <domain>      - perform SSL command on a domain
    dump                    - dump SSL certificates

    ";

    println!("{}", str);
}