# influxdb tsm存储引擎
## index-partition-indexFile的关系
- index包含多个partition
- partition包含多个indexFile

## index判断measurement是否存在
1. 并行遍历所有分区
2. 每个分区遍历所有indexFile
3. indexFile查找是否包含该measurement（干活的是indexFile😅）
> 貌似没有用bloom filter

## log entry
#### 存储结构
```
| 1 byte |   4 byte   |   4 byte   |     x byte    |   4 byte   |      y byte       |   4 byte   |     z byte     |   4 byte   |
--------------------------------------------------------------------------------------------------------------------------------
  flag     series id    name length       name        key length          key         value length       value        checksum

```
#### flag
- 删除measurement
- 删除tag key
- 删除tag value
- 添加series
