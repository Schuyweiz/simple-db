# SimpleDB: A Simple Database in Rust

SimpleDB is a minimalistic database implementation following a [tutorial](https://github.com/cstack/db_tutorial/tree/master) in Rust. This project is designed as an educational tool to demonstrate basic database concepts and Rust programming practices.

## Getting Started

These instructions will guide you through setting up the Rust environment and getting a copy of the project up and running on your local machine.

### Setting Up Rust

To run this project, you need to install Rust and its package manager, Cargo. Here's how you can set it up:

1. Install Rust: Follow the instructions on the official [Rust website](https://www.rust-lang.org/learn/get-started) to install Rustup, which will set up Rust and Cargo on your machine.

2. Verify the installation by running:

    ```bash
    rustc --version
    cargo --version
    ```

    You should see the installed versions of Rust and Cargo.

### Installing and Running the Project

1. Clone the repository:

    ```bash
    git clone https://github.com/your-username/simpledb.git
    ```

2. Navigate to the project directory:

    ```bash
    cd simpledb
    ```

3. To run the database, provide the name of the database file as a command-line argument:

    ```bash
    cargo run -- database_name.db
    ```

    Replace `database_name.db` with your preferred database file name. The program will create the database file if it doesn't exist.

### Usage

After running the project, you can interact with the database through the command line. The program accepts SQL-like commands and a special metadata command:

- **Metadata Command**:
  - `.exit`: Exits the database program.

- **SQL Commands**:
  - `select`: Retrieves and displays data from the database.
  - `insert id username email`: Inserts a new row into the database with the specified `id`, `username`, and `email`.

Example commands:

```sql
insert 1 user1 user1@example.com
select
.exit
