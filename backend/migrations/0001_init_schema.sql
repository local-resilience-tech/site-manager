-- Add migration script here
CREATE TABLE sites (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE network_configs (
    id INT PRIMARY KEY NOT NULL,
    network_name VARCHAR(255),
    bootstrap_node_id VARCHAR(64),
    bootstrap_node_ip4 VARCHAR(15)
);

CREATE TABLE site_configs (
    id INT PRIMARY KEY NOT NULL,
    this_site_id VARCHAR(36),
    private_key_hex VARCHAR(64),
    FOREIGN KEY (this_site_id) REFERENCES sites(id)
);