[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/7ZFsTk8-)

# Introduction
1. **What is Rust**?
    - Rust is a systems programming language focused on performance, reliability, and memory safety. 
    - It provides powerful abstractions, and its ownership model ensures that memory is managed safely without needing a garbage collector. 
    - Rust is popular for its ability to write fast and safe code, often used in systems programming, web development (via WebAssembly), and more.

2. **What is Cargo**?
    - Cargo is Rust's package manager and build system. 
    - It simplifies the management of Rust projects by handling dependencies, compiling code, running tests, and creating distributable packages. 
    - Cargo helps with managing libraries and packages, automating tasks like building, testing, and publishing, and it's essential for any Rust development workflow.

3. **What is Clippy**?
    - Clippy is a collection of lints (static code analysis tools) for Rust. 
    - It helps ensure that your code adheres to best practices and follows idiomatic Rust style.
    - Clippy can catch potential bugs, offer improvements for readability, and provide suggestions on how to make the code more efficient. 


# Hashassin
- Hashassin is a tool designed for generating and cracking password hashes.

# Project Overview
- This project is divided into **three main parts**, focusing on the core functionality of password generation, hash management, and password cracking using rainbow tables.
- [*Part 1: Password Generation and Hash Management*](#part-1) 
    - Generate random passwords
    - Create corresponding cryptographic hashes
    - Export (dump) the generated hashes to a file
- [*Part 2: Rainbow Table Creation and Hash Cracking*](#part-2) 
    - Generate a rainbow table based on selected hashing algorithms
    - Save (dump) the rainbow table to a file
    - Use the rainbow table to crack hashes and retrieve the original passwords
- [*Part 3: Server-Client System for Cracking Passwords*](#part-3) 
    - Develop a server-client architecture
    - Allow users to upload hash files and rainbow tables
    - Server processes the inputs and returns the cracked passwords

### Requirements
- Rust
- Cargo (Rust's package manager and build system)

### GRADING RUBRIC ATTEMPTED
> **Cargo fmt**
> - This command formats your Rust code according to the official style guidelines. 
> - It uses **rustfmt**, a tool that automatically formats Rust code to ensure consistent style across projects. 
> - You can run **cargo fmt** to format your code before committing or pushing to maintain readability and consistency.

> **cargo add <package>**
> - This command is used to add a new dependency to your project. 
> - It modifies the **Cargo.toml** file by adding the specified package and version to the dependencies section.

> **cargo add --package <package_name> <package>**
> - This command adds a dependency specifically to the named package within a workspace. 
> - If you're working with multiple packages in a single workspace, this allows you to specify which package should have the dependency added.
> cargo run
> - The cargo run command builds your project and then runs the resulting executable. 
> - It's a quick way to test and execute your Rust program without manually invoking the build process and then running the binary separately

> **cargo cippy**
> - Cargo clippy will provide suggestions and warnings for potential errors in your code, such as unused variables, unnecessary operations, and more

> **cargo check**
> - cargo check is a command that checks your Rust code for errors without actually compiling it into an executable or library.
> - It performs a quick syntax and type check to ensure that the code is correct, which is useful during development for catching issues early without waiting for a full compilation.

> **cargo doc**
> - cargo doc is a command in Rust's build system and package manager (Cargo) that generates documentation for your project.
> - It parses the Rust source code, including comments written using doc comments (/// for functions, structs, etc.), and produces HTML documentation
> - By default, it only generates documentation for public items, but you can use flags like **--document-private-items** to include private items as well. You can also use **--no-deps** to exclude documentation for dependencies

### DEPENDENCIES USED
- clap: For command-line argument parsing.

- hashassin-core: A core library for the hashing and cracking functionality.

- log: General logging utility for Rust.

- env_logger: Logging implementation for environment-based log levels.

- rand: Used for generating random passwords.

- crossbeam-channel: For multi-threaded communication in the application.

- md5: MD5 hashing algorithm.

- sha256: SHA-256 hashing algorithm.

- rs_sha3_512: SHA3-512 hashing algorithm.

- rayon: For parallel processing.

- scrypt: Scrypt hashing algorithm.

- thiserror: Simplifies error handling.

- hex: For encoding data to hexadecimal.

- sha2: For SHA-2 family algorithms like SHA-256.

- sha3: For SHA-3 family algorithms.

- rand_core: Core functionality for random number generation.

- password-hash: For password hashing and management.
- num: A collection of numeric types and traits for Rust, including bigint, complex, rational, range iterators, generic integers.

- ethereum-types: Ethereum-specific data types are primarily used within libraries and tools that interact

- dashmap: DashMap is an implementation of a concurrent associative array/hashmap in Rust. DashMap tries to implement an easy to use API similar to std::collections::HashMap with some slight changes to handle concurrency.

- tokio: Tokio is a runtime for writing reliable asynchronous applications with Rust. It provides async I/O, networking, scheduling, timers, and more.

### CLI Commands
### Part 1
#### **1.gen-passwords**

The gen-passwords command generates random passwords based on specified criteria and saves them to a file or outputs them to stdout.

> **Options:**
> - **--chars \<value>**
    1. Specifies the number of characters in each generated password. 
    2. This value must be <mark>greater than zero</mark> and should fit within an 8-bit range. 
    3. <mark>The default value is 4</mark>
> - **--out-file \<filename>**
    1. Specifies the file where generated passwords will be saved. 
    2. If this is not provided, the passwords will be output to stdout. If the file exists, it will be overwritten. If the file does not exist, it will be created.
> - **--threads \<number>**
    1. Defines the number of threads to use for generating passwords. 
    2. The value must be <mark>greater than zero</mark> and should <mark>not exceed the system’s maximum thread limit</mark>.
    3. <mark>The default is 1</mark> 
> - **--num \<number>**
    1. Specifies the number of passwords to generate. 
    2. This must be <mark>greater than zero</mark>. 
    3. The maximum value should be the system's limit on the size of an array. 

#### Example Usages:
1. Generate 10 random passwords with 8 characters each and print them to stdout
    > cargo run gen-passwords --chars 8 --num 10
2. Generate 100 passwords, save them to a file called passwords.txt, and use 4 threads
    > cargo run gen-passwords --chars 8 --num 100 --out-file passwords.txt

#### **2.gen-hashes**

The gen-hashes command is designed to generate hashes from a set of input passwords using specified algorithms. It offers multiple options to control the input, output, and the number of threads used during hash generation.

> **Options:**
> - **--in-file \<path>**
    1. Specifies the path to read plaintext passwords from. Each line in this file should contain one password.
    2. The length of the first password in the file will be assumed as the length of all passwords in the file 
> - **--out-file \<path>**
    1. Specifies the file where the generated hashes will be saved. The output file format is detailed below. 
    2. If the file already exists, it will be overwritten/truncated.
> - **--threads \<number>**
    1. Specifies the number of threads to use during hash generation. The value must be greater than 0.
    2. TThe maximum number of threads should not exceed the maximum length of an array on the system it’s being run on.
    3. <mark>The default is 1</mark> 
> - **--algorithm \<name>**
    1. Specifies the hashing algorithm to use. The available algorithms should be implemented in the program. Common options might include <mark> sha256, md5, sha3_512 and scrypt </mark>.

#### Example Usages:
1. Generate hashes from a file using sha256, saving to an output file with 4 threads
    > cargo run gen-hashes --in-file passwords.txt --out-file output.hashes --threads 4 --algorithm sha256

2. Generate 100 passwords, save them to a file called output.hashes, and use 1 threads
    > gen-hashes --in-file passwords.txt --out-file output.hashes --algorithm md5


> **Output File Format**
    1. **VERSION**: The first byte in the output file should contain the version number. Unless otherwise specified in future updates, this should always be 1.
    2. **ALGORITHM LENGTH**: The second byte contains the length of the algorithm name string (in ASCII encoding). This is a single byte representing the length of the string that follows.
    3. **ALGORITHM**: Starting at the 3rd byte, the algorithm name is encoded as an ASCII string (e.g., sha256, md5). The algorithm name must not be null-terminated.
    4. **PASSWORD LENGTH**: This byte will contain the length of each password used in the hash generation.
    5. **DATA**: The remaining bytes will contain the generated hashed passwords. Each hashed password should be zero-padded to align with the others.

#### Example of Output File Structure:
> VERSION (1 byte)   ALGORITHM LENGTH (1 byte)   ALGORITHM (ASCII string)   PASSWORD LENGTH (1 byte)   DATA (hashed passwords, zero-padded)


#### **3.dump-hashes**

The dump-hashes will take as input a generated hashes file and dump it to plaintext. dump-hashes has
one and only one parameter option.
> **Options:**
> - **--in-file \<path>**
    1. This takes a path to the file generated from gen-hashes that will be dumped to stdout.

#### Example Usages:
1. Generate dump hashes for a 100 passwords file from a file using md5, gives the below output using the command below 
    > cargo run --bin hashassin dump-hashes --in-file sample_outputs/100-scrypt.hashes

> **Output File Format**
    1. **VERSION**: $VERSION NUMBER”, where $VERSION NUMBER is the version number in the supplied input file.
    2. **ALGORITHM**: $ALGORITHM, where $ALGORITHM is the name of the algorithm as specified in the input file.
    3. **PASSWORD LENGTH**: $ALGORITHM”, where $ALGORITHM is the name of the algorithm as specified in the input file.
    4. **PASSWORD LENGTH**: $PASSWORD LENGTH”, where $PASSWORD LENGTH is the password length specified in the input file.

#### Example of Output File Structure:
> VERSION (1 byte)  
  ALGORITHM (md5) 
  PASSWORD LENGTH (1 byte)
  18c07a5177752088fe532ccb79a19963

### Part 2
#### **4.gen-rainbow-table**

The gen-rainbow-table commands generate a rainbow table with chains starting from a list of preexisting passwords given as a input file.

> **Options:**
> - **--in-file \<path>**
    1. Specifies the path to read plaintext passwords from. Each line in this file should contain one password.
    2. The length of the first password in the file will be assumed as the length of all passwords in the file 
> - **--out-file \<path>**
    1. Specifies the file where the generated rainbow table will be saved. The output file format is detailed below. 
    2. If the file already exists, it will be overwritten/truncated.
> - **--threads \<number>**
    1. Specifies the number of threads to use during hash generation. The value must be greater than 0.
    2. TThe maximum number of threads should not exceed the maximum length of an array on the system it’s being run on.
    3. <mark>The default is 1</mark> 
> - **--algorithm \<name>**
    1. Specifies the hashing algorithm to use. The available algorithms should be implemented in the program. Common options might include <mark> md5, sha256, sha3_512 and scrypt </mark>.
> - **--num-links \<number>**
    1. Specifies the number of links generated chains should have.
    2. Value should be greater than zero.
    3. <mark>The default is 5</mark>

> **Output File Format**
    1. **MAGIC WORD**: The first n bytes of the header will be a utf8 encoded string **“rainbowtable”** <mark>(all lower case)</mark>.
    2. **VERSION**: The next byte after the magic word in the output file contains the version number. 
    3. **ALGORITHM LENGTH**: The next byte contains the length of the algorithm name string (in ASCII encoding). This is a single byte representing the length of the string that follows.
    4. **ALGORITHM**: The next byte, the algorithm name is encoded as an ASCII string (e.g., sha256, md5). 
    5. **PASSWORD LENGTH**: This byte will contain the length of each password used in the hash generation.
    6. **CHARACTER SET SIZE**: The next 16 bytes of the file will be the character set size of pass
words in the rainbow table **(i.e., the radix we use in our reduction function that is <mark>95</mark>)**.
    7. **NUMBER OF LINKS**: The next 16 bytes will be the number of links in each chain of the
 rainbow file.
    8. **ASCII OFFSET**: The next byte will be any offset (from 0) that passwords in the rainbow
 table.

**Example Usages**:
1. Generate rainbow table with md5 algorithm, number of links = 5
    > cargo run gen-rainbow-table --in-file \<password-file> --out-file \<output-file-path>
2. Generate rainbow table with sha256 algorithm, number of links = 10 and threads = 10
    > cargo run gen-rainbow-table --in-file \<password-file> --out-file \<output-file-path> --algorithm \<algorithm_name> --threads 10 --num-links 10

#### **5.dump-rainbow-table**

The dump-rainbow-table commands print a human readable version of a rainbow table to the console.

> **Options:**
> - **--in-file \<path>**
    1. Which is the path to the rainbow table to dump. 
    2. This file is required and must be in gen-rainbow-table’s output format.

> **Console Output Format**
    1. Line 1: Hashassin Rainbow Table
    2. Line 1: VERSION: $VERSION_NUMBER
    3. Line 2: ALGORITHM: \$ALGORITHM
    4. Line 3: PASSWORD LENGTH: \$PASSWORD LENGTH
    5. Line 4: CHAR SET SIZE: CHAR SET SIZE
    6. Line 5: NUM LINKS: NUM_LINKS
    7. Line 6: ASCII OFFSET: ASCII OFFSET
    8. The remaining lines should be be the chains in the rainbow  Each line should be the start point of a chain and its corresponding end point, separated by a tab (\t)

**Example Usages**:
1. Dump rainbow table
    > cargo run dump-rainbow-table --in-file \<file_path/file_name>

#### **6.crack**

Given hash file from gen-hashes, produces any passwords for those hashes that are found in a pre-computed rainbow table.

> **Options:**
> - **--in-file \<path>**
    1. which specifies the path to read the rainbow table from. 
> - **--out-file \<path>**
    1. if present, will write the output of the command to the specified file, with one pair of hash hex encoded and corresponding password separated by the tab character, per line,. If not present, results should be written to stdout.
> - **--threads \<num>**
    1. The number of threads to use to crack passwords. 
    2. This value must be greater than zero and the maximum value should be the maximum length of an array on whatever system the program is being run on.
    3. threads must have a default value of 1.
> - **--hashes \<path>**
    1. which is a path to a set of hashes to crack. 
    2. The input path must (by default) have hashes in the gen-hashes

**Console Output Format**
   

**Example Usages**:
1. Dump rainbow table
    > cargo run dump-rainbow-table --in-file \<file_path/file_name>

### Part 3
- Added two more crate **server** and **client**. Server launches a network server that responds to requests from clients. This server will accept two commands

#### **7. Server**
> **Options:**
> - **--bind \<address>**
    1. Which specifies the ip address your server should bind to.--bind should default to 127.0.0.1
> - **--port \<port>**
    1. Which specifies the port youare server should bind to. This should be a non-zero u16.
    2. A default value of 2025.
> - **--compute-threads \<num>**
    1. The total number of threads that are available to the server to crack passwords. 
    2. This value will be greater than zero and the maximum value should be the maximum length of an array on whatever system the program is being run on.
    3. Have a default value of 1.
> - **--async-threads \<num>**
    1. Which will set the number of threads that your async runtime uses for tasks.
    2. This option will have default of 1.
> - **--cache-size \<num>**
    1. Is an optional value that indicates that already cracked passwords should be cached, and that the cache should be no larger than the maximum value of an i32 bytes.

**Example Usages**:
1. Start Server
    > - cargo run server --bind \<address> --port \<port>

#### **8.Client**

Upload which will upload a rainbow table to the server and crack which will have the server attempt to crack a supplied hashes file.

##### **A.Upload** 
- Upload which will accept rainbow table from a client that can be used for cracking passwords. 
- Uploaded rainbow tables will persist for at least as long as the server is running. I.e., multiple  rainbow tables can be uploaded at the same time and all these rainbow tables will be available to any clients that want to crack them.

> **Options:**
> - **--server \<address>**
    1. which will be the address of the server to connect to. E.g., “127.0.0.1:2025”
> - **--in-file \<path>**
    1. which is the rainbow table file to upload to the server
> - **--name \<name>**
    1. Which is a name that can be associated with the rainbow table being uploaded.

> **Requests Format:**
> **1. Upload**: consists of a header with a few fields and a payload section
> 1. MAGIC WORD: Thefirst n bytes of an upload command should be a utf8 encoded string “upload” (all lower case).
> 2. VERSION: The next byte after the end of the magic word must be a version number. This should always be 1.
> 3. NAME LENGTH: The byte following the version number should be the length of the name of the upload.
> 4. NAME: The NAME LENGTH bytes following the name of the upload should be the utf8 encoded name of the upload.
> 5. PAYLOAD SIZE the next 8 bytes should be the length of the payload in bytes. E.g., a value of 1 here would mean that the payload is one byte total.
> 6. PAYLOAD: the next PAYLOAD SIZE bytes should be a rainbow table, in the same format as specified for part 2.

> **2. Crack**: The format for crack consists of a header with a few fields and a payload section.
> 1. MAGIC WORD: The first n bytes of an crack command should be a utf8 encoded string “crack”
 (all lower case).
> 2. VERSION: The next byte after the end of the magic word must be a version number. Unless
 otherwise specified in a future update, this should always be 1.
> 3. PAYLOAD SIZE the next 8 bytes should be the length of the payload.
> 4. PAYLOAD: the next PAYLOAD SIZE bytes should be a hashes file, in the same format as specified for part 1 and 2.

**Example Usages**:
1. Upload Rainbow table
    > - cargo run client upload --server 127.0.0.1:2025 --in-file .\sample_rainbow.rt --name rainbow_table_1

##### **B.Crack** 
- crack which will accept a hashes file from a client and attempt to crack them. 
- When a client requests a crack, all rainbow tables that have been uploaded since the server was running should be used to attempt to crack the hashes

> **Options:**
> - **--server \<address>**
    1. which will be the address of the server to connect to. E.g., “127.0.0.1:2025”
> - **--in-file \<path>**
    1. which is input hashes file in the same format as existing hash files.
> - **--out-file \<path>**
    1. which is a optional and is the path to a file to save output from the server to. if not present, output should be to stdout.

**Example Usages**:
1. Crack Password
    > - cargo run client crack --server 127.0.0.1:2025 --in-file hashes.hashes --out-file cracked.txt

## 9. PERFORMANCE REPORTING

See PERFORMANCE.md for detailed analysis on:

 - Password length scaling.

 - Algorithm performance.

 - Thread scaling.

 - Network server-client load balancing performance.

## 10. PROJECT STRUCTURE

Hashassin/
├── cli/                      # Command-line interface
├── core/                     # Core hashing, reduction, and cracking logic
├── client/                   # Client-side upload/crack implementations
├── server/                   # Server-side TCP server implementation
├── PERFORMANCE.md            # Performance analysis and results
├── CREDITS.md                # Contributions
├── HONESTY.md                # Academic honesty statement
└── README.md                 # This file

## 11. USAGE SUMMARY

Generate passwords and hashes

Dump and inspect hash/rainbow files

Build and deploy a networked cracking server

Upload and crack hashes remotely using the client

Evaluate performance with multi-threaded and networked execution