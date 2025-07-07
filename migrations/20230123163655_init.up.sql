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
