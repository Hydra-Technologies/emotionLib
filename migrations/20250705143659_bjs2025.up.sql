CREATE TABLE category(
    id INT PRIMARY KEY NOT NULL,
    group INT NOT NULL
);

CREATE TABLE category_group(
    id INT PRIMARY KEY NOT NULL,
    name VARCHAR(128)
);

CREATE TABLE mand_category(
    age INT NOT NULL,
    gender CHAR NOT NULL,
    category_id INT NOT NULL,
);

CREATE TABLE form_vars(
    category_id INT,
    gender CHAR,
    a DOUBLE,
    c DOUBLE
);

CREATE TABLE points_eval(
    age INT,
    gender CHAR,
    winner INT,
    honor INT
);
