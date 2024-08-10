# API 清单

## `dates?month={$m:%d}&year={$y:%d}`
返回日期对应的次数, 未经任何修改

```sql
SELECT
  day,
  COUNT(*) AS times
FROM
  main
WHERE
  year=$y
  AND month=$m
GROUP BY
  day
;
```

## `fixed_dates?offset={$f:%d}&month={$m:%d}&year={$y:%d}`
返回每个调整过的日期对应的次数, 其中某一天的起点被定义为该日0时加上offset指定的偏移量

```sql
SELECT
  ordinal,
  time
FROM
  main
INNER JOIN
  extrainfo AS ex
ON
  main.rowid=ex.id
WHERE
  ordinal BETWEEN $0 AND $1
ORDER BY
  ordinal ASC
;
-- 查询结果需进一步整理得到次数, 可以保证结果中的日期按升序排列
```
