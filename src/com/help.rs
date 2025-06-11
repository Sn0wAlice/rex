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

pdf <command> <file_path>   - perform PDF command on a file
    extract                 - extract metadata and images from a PDF file

net <command>               - perform network command on a domain
    log                     - show network logs (requires root privileges)

    ";

    println!("{}", str);
}