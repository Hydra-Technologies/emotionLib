CREATE TABLE kategorien(
    id INTEGER,
    name VARCHAR(255),
    einheit CHAR,
    maxVers INT, 
    digits_before INT,
    digits_after INT,
    PRIMARY KEY (id)
);

CREATE TABLE schueler(
    id INT,
    external_id INT,
    fName VARCHAR(255), 
    lName VARCHAR(255), 
    klasse VARCHAR(10),
    gesch CHAR,
    birth_year INT,
    age INT,
    aufsicht boolean, 
    llTime INT,
    llkey VARCHAR(255),
    PRIMARY KEY(id)
);


CREATE TABLE versuch(
    id INT NOT NULL,
    aufsichtId VARCHAR(10) NOT NULL,
    schuelerId INT NOT NULL, 
    kategorieId INT NOT NULL, 
    wert DOUBLE NOT NULL, 
    mTime INT NOT NULL, 
    isReal boolean NOT NULL,
    PRIMARY KEY (id), 
    FOREIGN KEY (schuelerId) REFERENCES schueler(id),
    FOREIGN KEY (kategorieId) REFERENCES kategorien(id)
);

CREATE TABLE event (
    id VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE tmp_user (
    id VARCHAR(10) PRIMARY KEY NOT NULL,
    api_key VARCHAR(512) NOT NULL,
    vouched boolean NOT NULL,
    time_of_creation INT NOT NULL,
    time_of_activation INT,
    last_refresh INT NOT NULL,
    event_id VARCHAR(10),
    FOREIGN KEY (event_id) REFERENCES event(id)
);

CREATE TABLE user_session (
    api_key VARCHAR(512) NOT NULL,
    time_of_creation INT NOT NULL,
    last_refresh INT NOT NULL,
    PRIMARY KEY (api_key)
);

CREATE TABLE category(
    id INT PRIMARY KEY NOT NULL,
    category_group_id INT NOT NULL,
    running BOOLEAN NOT NULL,
    distance INT
);

CREATE TABLE category_group(
    id INT PRIMARY KEY NOT NULL,
    name VARCHAR(128)
);

CREATE TABLE mand_category(
    age INT NOT NULL,
    gender CHAR NOT NULL,
    category_id INT NOT NULL,
    gold DOUBLE NOT NULL,
    silver DOUBLE NOT NULL,
    bronze DOUBLE NOT NULL
);

CREATE TABLE form_vars(
    category_id INT NOT NULL,
    gender CHAR NOT NULL,
    a DOUBLE NOT NULL,
    c DOUBLE NOT NULL
);

CREATE TABLE points_eval(
    age INT NOT NULL,
    gender CHAR NOT NULL,
    winner INT NOT NULL,
    honor INT NOT NULL
);
