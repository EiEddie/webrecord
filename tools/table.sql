CREATE TABLE main (
	id    INTEGER NOT NULL PRIMARY KEY,
	year  INTEGER NOT NULL
	       DEFAULT(CAST(strftime('%Y', 'now', 'localtime') AS INTEGER)),
	month INTEGER NOT NULL
	       DEFAULT(CAST(strftime('%m', 'now', 'localtime') AS INTEGER))
	       CHECK(month BETWEEN 1 AND 12),
	day   INTEGER NOT NULL
	       DEFAULT(CAST(strftime('%d', 'now', 'localtime') AS INTEGER))
	       CHECK(day BETWEEN 1 AND 31),
	time  TEXT
	       DEFAULT(NULL)
	       CHECK(length(time)=5 OR time IS NULL)
);


CREATE TABLE extra (
	main_id  INTEGER NOT NULL,
	datetime TEXT    NOT NULL,
	ordinal  INTEGER,
	PRIMARY KEY (main_id),
	FOREIGN KEY (main_id) REFERENCES main(id)
) WITHOUT ROWID;


CREATE TRIGGER make_extra_info
AFTER INSERT ON main
FOR EACH ROW
BEGIN
	INSERT INTO extra(main_id, ordinal, datetime)
	VALUES (NEW.id, NULL,
	        printf('%d-%02d-%02d %s', NEW.year, NEW.month, NEW.day, IFNULL(NEW.time, '12:00')));

	UPDATE extra
	   SET ordinal = CAST(julianday(datetime, 'start of day') - julianday('1970-01-01') AS INTEGER)
	 WHERE main_id=NEW.id;
END;


CREATE TRIGGER update_extra_info
AFTER UPDATE ON main
FOR EACH ROW
BEGIN
	UPDATE extra
	   SET datetime = printf('%d-%02d-%02d %s', NEW.year, NEW.month, NEW.day, IFNULL(NEW.time, '12:00'))
	 WHERE main_id=NEW.rid;

	UPDATE extra
	   SET ordinal = CAST(julianday(datetime, 'start of day') - julianday('1970-01-01') AS INTEGER)
	 WHERE main_id=NEW.id;
END;
