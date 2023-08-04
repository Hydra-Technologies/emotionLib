CREATE TABLE katGroups(
    id INT,
    name VARCHAR(255),
    numPflicht INT NOT NULL,
    forEDay BOOLEAN DEFAULT true,
    PRIMARY KEY (id)
);

CREATE TABLE kategorien(
    id INT,
    name VARCHAR(255),
    lauf BOOLEAN, 
    einheit CHAR, 
    maxVers INT, 
    messungsForm VARCHAR(255),
    kateGroupId INT,
    FOREIGN KEY (kateGroupId) REFERENCES katGroups(id),
    PRIMARY KEY (id)
);


CREATE TABLE formVars(
    katId INT NOT NULL,
    gesch CHAR,
    a DOUBLE, 
    c DOUBLE,
    PRIMARY KEY (katId, gesch),
    FOREIGN KEY (katId) REFERENCES kategorien(id)
);

CREATE TABLE ageGroups(
    age INT, 
    gesch CHAR, 
    gold INT, 
    silber INT, 
    PRIMARY KEY (age, gesch)
);

CREATE TABLE schueler(
    id int, 
    fName VARCHAR(255), 
    lName VARCHAR(255), 
    klasse VARCHAR(10),
    bDay VARCHAR(100),
    gesch CHAR,
    age INT,
    aufsicht boolean, 
    llTime INT,
    llkey VARCHAR(255),
    PRIMARY KEY(id), 
    FOREIGN KEY (age, gesch) REFERENCES ageGroups(age, gesch)
);


CREATE TABLE versuch(
    id INT NOT NULL,
    aufsichtId VARCHAR(10) NOT NULL,
    schuelerId INT NOT NULL, 
    kategorieId INT NOT NULL, 
    wert DOUBLE, 
    punkte INT, 
    mTime INT, 
    isReal boolean,
    PRIMARY KEY (id), 
    FOREIGN KEY (schuelerId) REFERENCES schueler(id),
    FOREIGN KEY (kategorieId) REFERENCES kategorien(id)
);

CREATE TABLE bjsKat (
    age INT,
    gesch CHAR,
    katId INT,
    FOREIGN KEY (age, gesch) REFERENCES ageGroups(age, gesch),
    PRIMARY KEY (age, gesch, katId)
);

CREATE TABLE dosbKat(
    age INT,
    gesch CHAR,
    katId INT,
    gold DOUBLE,
    silber DOUBLE,
    bronze DOUBLE,
    FOREIGN KEY (age, gesch) REFERENCES ageGroups(age, gesch),
    PRIMARY KEY (age, gesch, katId)
);

CREATE TABLE loginKeys(
    aufsichtId INT,
    token VARCHAR(512),
    buildTime INT,
    PRIMARY KEY (aufsichtId, token),
    FOREIGN KEY (aufsichtId) REFERENCES schueler(id)
);

INSERT INTO ageGroups(age, gesch, silber, gold) VALUES
    (10, 'w', 625, 825),
    (11, 'w', 700, 900),
    (12, 'w', 775, 975),
    (13, 'w', 825, 1025),
    (14, 'w', 850, 1050),
    (15, 'w', 875, 1075),
    (16, 'w', 900, 1100),
    (17, 'w', 925, 1125),
    (18, 'w', 950, 1150),
    (19, 'w', 950, 1150),
    (20, 'w', 950, 1150),

    (10, 'm', 600, 775),
    (11, 'm', 675, 875),
    (12, 'm', 750, 975),
    (13, 'm', 825, 1050),
    (14, 'm', 900, 1125),
    (15, 'm', 975, 1225),
    (16, 'm', 1050, 1325),
    (17, 'm', 1125, 1400),
    (18, 'm', 1200, 1475),
    (19, 'm', 1275, 1550),
    (20, 'm', 1275, 1550);

INSERT INTO katGroups(id, name, numPflicht) VALUES
    (1, 'Sprint', 1),
    (2, 'Sprung', 1),
    (3, 'Wurf/Stoß', 1),
    (4, 'Ausdauer', 1);

INSERT INTO kategorien(id, name, einheit, maxVers, lauf, messungsForm, kateGroupId) VALUES
    (1, '50m', 's', 1, true, '{2;s},{2;cs}s', 1),
    (2, '75m', 's', 1, true, '{2;s},{2;cs}s', 1),
    (3, '100m', 's', 1, true, '{2;s},{2;cs}s', 1),
    (4, '800m', 's', 1, true, '{1;min}min {2;s}s', 4),
    (5, '2000m', 's', 1, true, '{2;min}min {2;s}s', 4),
    (6, 'Hochsprung', 'm', 3, false, '{1;m},{2;cm}', 2),
    (7, 'Weitsprung', 'm', 3, false, '{1;m},{2;cm}', 2),
    (8, 'Kugelstoß', 'm', 3, false, '{2;m},{1;dm}', 3),
    (9, 'Schleuderball', 'm', 3, false, '{2;m},{1;dm}', 3),
    (10, '200g Wurf', 'm', 3, false, '{2;m},{1;dm}', 3),
    (11, '80g Wurf', 'm', 3, false, '{2;m},{1;dm}', 3);

INSERT INTO formVars(katId, gesch, a, c) VALUES
    (1, 'w', 3.64800, 0.00660),
    (1, 'm', 3.79000, 0.00690),

    (2, 'w', 3.99800, 0.00660),
    (2, 'm', 4.10000, 0.00664),

    (3, 'w', 4.00620, 0.00656),
    (3, 'm', 4.34100, 0.00676),

    (4, 'w', 2.02320, 0.00647),
    (4, 'm', 2.32500, 0.00644),

    (5, 'w', 1.80000, 0.00540),
    (5, 'm', 1.78400, 0.00600),

    (6, 'w', 0.88070, 0.00068),
    (6, 'm', 0.84100, 0.00080),

    (7, 'w', 1.09350, 0.00208),
    (7, 'm', 1.15028, 0.00219),

    (8, 'w', 1.27900, 0.00398),
    (8, 'm', 1.42500, 0.00370),

    (9, 'w', 1.08500, 0.00921),
    (9, 'm', 1.59500, 0.009125),

    (10, 'w', 1.41490, 0.01039),
    (10, 'm', 1.93600, 0.01240),

    (11, 'w', 2.02320, 0.00874),
    (11, 'm', 2.80000, 0.01100);

INSERT INTO dosbKat(katId, age, gesch, bronze, silber, gold) VALUES
    (4, 11, 'w', 320, 280, 240),
    (4, 13, 'w', 310, 265, 225),
    (4, 15, 'w', 300, 260, 215),
    (4, 17, 'w', 290, 245, 205),

    (11, 11, 'w', 11, 15, 18),
    (11, 13, 'w', 15, 18, 22),
    (10, 15, 'w', 20, 24, 27),
    (10, 17, 'w', 24, 27, 31),

    (1, 11, 'w', 11, 10.1, 9.1),
    (1, 13, 'w', 10.6, 9.6, 8.5),
    (3, 15, 'w', 18.6,17,15.5),
    (3, 17, 'w', 17.6,16.3,15),

    (6, 11, 'w', 0.8, 0.9, 1),
    (6, 13, 'w', 0.9, 1, 1.1),
    (6, 15, 'w', 0.95, 1.05, 1.15),
    (6, 17, 'w', 1.05, 1.15, 1.25),

    (7, 11, 'w', 2.3, 2.6, 2.9),
    (7, 13, 'w', 2.8, 3.1, 3.4),
    (7, 15, 'w', 3.2, 3.5, 3.8),
    (7, 17, 'w', 3.4, 3.7, 4);

INSERT INTO dosbKat(katId, age, gesch, bronze, silber, gold) SELECT katId, (age+1) as age, gesch, bronze, silber, gold FROM dosbKat;

INSERT INTO dosbKat(katId, age, gesch, bronze, silber, gold) VALUES
    (4, 11, 'm', 320, 280, 240),
    (4, 13, 'm', 310, 265, 225),
    (4, 15, 'm', 300, 260, 215),
    (4, 17, 'm', 290, 245, 205),

    (11, 11, 'm', 11, 15, 18),
    (11, 13, 'm', 15, 18, 22),
    (10, 15, 'm', 20, 24, 27),
    (10, 17, 'm', 24, 27, 31),

    (1, 11, 'm', 11, 10.1, 9.1),
    (1, 13, 'm', 10.6, 9.6, 8.5),
    (3, 15, 'm', 18.6,17,15.5),
    (3, 17, 'm', 17.6,16.3,15),

    (6, 11, 'm', 0.8, 0.9, 1),
    (6, 13, 'm', 0.9, 1, 1.1),
    (6, 15, 'm', 0.95, 1.05, 1.15),
    (6, 17, 'm', 1.05, 1.15, 1.25),

    (7, 11, 'm', 2.3, 2.6, 2.9),
    (7, 13, 'm', 2.8, 3.1, 3.4),
    (7, 15, 'm', 3.2, 3.5, 3.8),
    (7, 17, 'm', 3.4, 3.7, 4);

INSERT INTO bjsKat(age, gesch, katId) VALUES
    (10, 'w', 1),
    (10, 'w', 6),
    (10, 'w', 7),
    (10, 'w', 11),
    (10, 'w', 4),
    (10, 'w', 5),

    (11, 'w', 1),
    (11, 'w', 6),
    (11, 'w', 7),
    (11, 'w', 11),
    (11, 'w', 4),
    (11, 'w', 5),

    (12, 'w', 1),
    (12, 'w', 6),
    (12, 'w', 7),
    (12, 'w', 10),
    (12, 'w', 11),
    (12, 'w', 4),
    (12, 'w', 5),

    (13, 'w', 1),
    (13, 'w', 2),
    (13, 'w', 6),
    (13, 'w', 7),
    (13, 'w', 10),
    (13, 'w', 11),
    (13, 'w', 4),
    (13, 'w', 5),

    (14, 'w', 2),
    (14, 'w', 6),
    (14, 'w', 7),
    (14, 'w', 10),
    (14, 'w', 4),
    (14, 'w', 5),

    (15, 'w', 2),
    (15, 'w', 3),
    (15, 'w', 6),
    (15, 'w', 7),
    (15, 'w', 10),
    (15, 'w', 4),
    (15, 'w', 5),

    (16, 'w', 3),
    (16, 'w', 6),
    (16, 'w', 7),
    (16, 'w', 10),
    (16, 'w', 4),

    (17, 'w', 3),
    (17, 'w', 6),
    (17, 'w', 7),
    (17, 'w', 10),
    (17, 'w', 4),

    (18, 'w', 3),
    (18, 'w', 6),
    (18, 'w', 7),
    (18, 'w', 10),
    (18, 'w', 4);

INSERT INTO bjsKat(age, gesch, katId) VALUES
    (10, 'm', 1),
    (10, 'm', 6),
    (10, 'm', 7),
    (10, 'm', 11),
    (10, 'm', 4),
    (10, 'm', 5),

    (11, 'm', 1),
    (11, 'm', 6),
    (11, 'm', 7),
    (11, 'm', 11),
    (11, 'm', 10),
    (11, 'm', 4),
    (11, 'm', 5),

    (12, 'm', 1),
    (12, 'm', 6),
    (12, 'm', 7),
    (12, 'm', 10),
    (12, 'm', 4),
    (12, 'm', 5),

    (13, 'm', 1),
    (13, 'm', 2),
    (13, 'm', 6),
    (13, 'm', 7),
    (13, 'm', 10),
    (13, 'm', 4),
    (13, 'm', 5),

    (14, 'm', 2),
    (14, 'm', 6),
    (14, 'm', 7),
    (14, 'm', 10),
    (14, 'm', 4),
    (14, 'm', 5),

    (15, 'm', 2),
    (15, 'm', 3),
    (15, 'm', 6),
    (15, 'm', 7),
    (15, 'm', 10),
    (15, 'm', 4),
    (15, 'm', 5),

    (16, 'm', 3),
    (16, 'm', 6),
    (16, 'm', 7),
    (16, 'm', 10),
    (16, 'm', 4),

    (17, 'm', 3),
    (17, 'm', 6),
    (17, 'm', 7),
    (17, 'm', 10),
    (17, 'm', 4),

    (18, 'm', 3),
    (18, 'm', 6),
    (18, 'm', 7),
    (18, 'm', 10),
    (18, 'm', 4);

INSERT INTO schueler(id, fName, lName, klasse, bDay, gesch, aufsicht) VALUES ('4321', 'Brian2', 'aka Jesus', '5A', '2004-05-14', 'm', true);
INSERT INTO schueler(id, fName, lName, klasse, bDay, gesch, aufsicht) VALUES ('654321', 'Sir', 'Lancelot2', 'Q1', '2004-05-14', 'm', true);
INSERT INTO schueler(id, fName, lName, klasse, bDay, age, gesch, aufsicht) VALUES ('1234', 'Franz2', 'Peterson', '5A', '2007-12-23', '15' , 'w', false);
INSERT INTO schueler(id, fName, lName, klasse, bDay, gesch, aufsicht) VALUES ('3809', 'Frederik2', 'Folkers', 'Q2', '2005-06-20', 'm', true);