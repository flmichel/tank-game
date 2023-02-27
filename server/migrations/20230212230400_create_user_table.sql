-- Create account Table
CREATE TABLE account (
   name TEXT NOT NULL,
   email TEXT NOT NULL UNIQUE,
   password_hash TEXT NOT NULL,
   PRIMARY KEY (name)
);

CREATE TABLE video_game (
   name TEXT NOT NULL,
   description TEXT NOT NULL,
   PRIMARY KEY (name)
)