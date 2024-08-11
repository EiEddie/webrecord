CREATE TABLE 'main' (
  'year' INTEGER NOT NULL DEFAULT (CAST(substr(CURRENT_DATE, 1, 4) AS INTEGER)),
  'month' INTEGER NOT NULL DEFAULT (CAST(substr(CURRENT_DATE, 6, 2) AS INTEGER)) CHECK (month >= 1 AND month <= 12),
  'day' INTEGER NOT NULL DEFAULT (CAST(substr(CURRENT_DATE, 9, 2) AS INTEGER)) CHECK (day >= 1 AND day <= 31),
  'time' TEXT DEFAULT NULL CHECK (length(time)=5 OR time IS NULL)
);


CREATE TABLE 'extrainfo' (
  'id' INTEGER NOT NULL,
  'datetime' TEXT NOT NULL,
  'ordinal' INTEGER,
  PRIMARY KEY (id),
  FOREIGN KEY (id) REFERENCES main(rowid)
) WITHOUT ROWID;


CREATE TRIGGER 'makeinfo'
AFTER INSERT ON 'main'
FOR EACH ROW
BEGIN
  INSERT INTO
    extrainfo(id, datetime, ordinal)
  VALUES
    (NEW.rowid, printf('%d-%02d-%02d %s', NEW.year, NEW.month, NEW.day, IFNULL(NEW.time, '12:00')), NULL)
  ;

  UPDATE
    extrainfo
  SET
    ordinal=CAST(julianday(datetime, 'start of day') - julianday('1970-01-01') AS INTEGER)
  WHERE
    id=NEW.rowid
  ;
END;


CREATE TRIGGER 'updateinfo'
AFTER UPDATE ON 'main'
FOR EACH ROW
BEGIN
  UPDATE
    extrainfo
  SET
    datetime=printf('%d-%02d-%02d %s', NEW.year, NEW.month, NEW.day, IFNULL(NEW.time, '12:00'))
  WHERE
    id=NEW.rowid
  ;

  UPDATE
    extrainfo
  SET
    ordinal=CAST(julianday(datetime, 'start of day') - julianday('1970-01-01') AS INTEGER)
  WHERE
    id=NEW.rowid
  ;
END;

