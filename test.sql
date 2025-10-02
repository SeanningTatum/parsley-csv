CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE
);

INSERT INTO users VALUES(1, 'Alice Smith', 'alice@example.com');
INSERT INTO users VALUES(2, 'Bob Johnson', 'bob@example.com');
INSERT INTO users VALUES(3, 'Charlie Brown', 'charlie@example.com');

CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL,
    category TEXT
);

INSERT INTO products VALUES(1, 'Laptop', 999.99, 'Electronics');
INSERT INTO products VALUES(2, 'Coffee Mug', 12.50, 'Kitchen');
INSERT INTO products VALUES(3, 'Notebook', 5.99, 'Office');
INSERT INTO products VALUES(4, 'Headphones', 79.99, 'Electronics');
