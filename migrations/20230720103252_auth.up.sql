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
