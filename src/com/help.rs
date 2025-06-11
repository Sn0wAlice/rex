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

reg <command> <args>        - perform registry command
    systemd <service_name>  - manage systemd services
        extract             - extract logs from systemd journal
               -group       - extract logs from systemd journal for a specific group
               -n <number>  - specify number of logs to extract (default: 100)
        scan                - scan systemd services for issues
               -n <number>  - specify number of services to scan (default: 100)
        sshfail             - extract failed SSH login attempts
        deepscan            - perform a deep scan of systemd services
               --save       - save the scan results to a file

    ";

    println!("{}", str);
}