-- Add up migration script here
CREATE TABLE school (
    id INT PRIMARY KEY,
    name VARCHAR(255),
    dbName VARCHAR(255)
);

CREATE TABLE groups (
    id INT PRIMARY KEY,
    name VARCHAR(255)
);

CREATE TABLE endpoints (
    method VARCHAR(255),
    path VARCHAR(255),
    g_level INT,
    PRIMARY KEY (method, path)
);

CREATE TABLE keys (
    id VARCHAR(10) PRIMARY KEY NOT NULL,
    group_id INT NOT NULL,
    school_id INT NOT NULL,
    api_key VARCHAR(512) NOT NULL,
    time_of_creation INT,
    vouch_id VARCHAR(10)
);

INSERT INTO school VALUES (1, 'demoSchool1', 'emotion1'), (2, 'demoSchool2', 'emotion2');
INSERT INTO groups VALUES (1, 'root'), (2, 'admin'), (3, 'teacher'), (4, 'student');
INSERT INTO endpoints VALUES
    ('GET', '/schueler/', 3),
    ('GET', '/schueler/{}', 3),
    ('POST', '/schueler/{}', 3),
    ('GET', '/schueler/{}/kategorien/dosb', 3),
    ('GET', '/schueler/{}/kategorien/bjs', 3),
    ('GET', '/schueler/{}/kategorien', 3),
    ('GET', '/schueler/{}/kategorien/{}', 3),
    ('GET', '/schueler/{}/kategorien/{}/top', 3),
    ('GET', '/schueler/{}/top', 3),

    ('GET', '/versuch/{}', 3),
    ('PUT', '/versuch/{}', 3),

    ('GET', '/kategorie', 3),
    ('GET', '/kategorie/{}', 3);
INSERT INTO keys VALUES ('Emo1', 1, 1, 'ABC', 0, 0);
INSERT INTO keys VALUES ('Emo2', 1, 2, 'CBA', 0, 0);
