CREATE TABLE page (
  id PRIMARY KEY,
  content
);

CREATE TABLE post (
  slug PRIMARY KEY,
  timestamp INTEGER,
  title,
  tag,
  content
);

CREATE UNIQUE INDEX idx_post_ts ON post(timestamp DESC);

ALTER TABLE post ADD COLUMN published INTEGER DEFAULT FALSE;
ALTER TABLE page ADD COLUMN title;
