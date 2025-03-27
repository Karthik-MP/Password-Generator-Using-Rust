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
- This project, we will focus on the basic functionality for generating random passwords and managing them.

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


### CLI Commands

**1.gen-passwords**

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

### Example Usages:
1. Generate 10 random passwords with 8 characters each and print them to stdout
    > cargo run gen-passwords --chars 8 --num 10
2. Generate 100 passwords, save them to a file called passwords.txt, and use 4 threads
    > cargo run gen-passwords --chars 8 --num 100 --out-file passwords.txt

**2.gen-hashes**

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

### Example Usages:
1. Generate hashes from a file using sha256, saving to an output file with 4 threads
    > cargo run gen-hashes --in-file passwords.txt --out-file output.hashes --threads 4 --algorithm sha256

2. Generate 100 passwords, save them to a file called output.hashes, and use 1 threads
    > gen-hashes --in-file passwords.txt --out-file output.hashes --algorithm md5


> **Output File Format**
> - **--in-file \<path>**
    1. **VERSION**: The first byte in the output file should contain the version number. Unless otherwise specified in future updates, this should always be 1.
    2. **ALGORITHM LENGTH**: The second byte contains the length of the algorithm name string (in ASCII encoding). This is a single byte representing the length of the string that follows.
    3. **ALGORITHM**: Starting at the 3rd byte, the algorithm name is encoded as an ASCII string (e.g., sha256, md5). The algorithm name must not be null-terminated.
    4. **PASSWORD LENGTH**: This byte will contain the length of each password used in the hash generation.
    5. **DATA**: The remaining bytes will contain the generated hashed passwords. Each hashed password should be zero-padded to align with the others.

### Example of Output File Structure:
> VERSION (1 byte)   ALGORITHM LENGTH (1 byte)   ALGORITHM (ASCII string)   PASSWORD LENGTH (1 byte)   DATA (hashed passwords, zero-padded)


**3.dump-hashes**

The dump-hashes will take as input a generated hashes file and dump it to plaintext. dump-hashes has
one and only one parameter option.
> **Options:**
> - **--in-file \<path>**
    1. This takes a path to the file generated from gen-hashes that will be dumped to stdout.

### Example Usages:
1. Generate dump hashes for a 100 passwords file from a file using md5, gives the below output using the command below 
    > cargo run --bin hashassin dump-hashes --in-file sample_outputs/100-scrypt.hashes

> **Output File Format**
> - **--in-file \<path>**
    1. **VERSION**: $VERSION NUMBER”, where $VERSION NUMBER is the version number in the supplied input file.
    2. **ALGORITHM**: $ALGORITHM, where $ALGORITHM is the name of the algorithm as specified in the input file.
    3. **PASSWORD LENGTH**: $ALGORITHM”, where $ALGORITHM is the name of the algorithm as specified in the input file.
    4. **PASSWORD LENGTH**: $PASSWORD LENGTH”, where $PASSWORD LENGTH is the password length specified in the input file.

### Example of Output File Structure:
> VERSION (1 byte)  
  ALGORITHM (md5) 
  PASSWORD LENGTH (1 byte)
  18c07a5177752088fe532ccb79a19963
