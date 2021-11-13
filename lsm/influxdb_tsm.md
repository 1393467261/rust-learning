# influxdb tsm存储引擎
## index-partition-indexFile的关系
- index包含多个partition
- partition包含多个indexFile

## index判断measurement是否存在
1. 并行遍历所有分区
2. 每个分区遍历所有indexFile
3. indexFile查找是否包含该measurement（干活的是indexFile😅）
> 貌似没有用bloom filter
