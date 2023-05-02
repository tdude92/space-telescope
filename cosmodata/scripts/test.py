from datetime import date, datetime

from pyspark.sql import SparkSession

spark = SparkSession.builder.getOrCreate()

tableName = "hudi_trips_cow"
basePath = "file:///tmp/hudi_trips_cow"

hudi_options = {
    "hoodie.table.name": tableName,
    "hoodie.datasource.write.recordkey.field": "name",
    "hoodie.datasource.write.partitionpath.field": "birthday",
    "hoodie.datasource.write.table.name": tableName,
    "hoodie.datasource.write.operation": "upsert",
    "hoodie.datasource.write.precombine.field": "checkin",
    "hoodie.upsert.shuffle.parallelism": 2,
    "hoodie.insert.shuffle.parallelism": 2,
}

# write
df = spark.createDataFrame(
    [
        (69.0, "Joe", date(2000, 1, 1), datetime(2000, 1, 1, 12, 0)),
        (87.0, "Don", date(2000, 2, 1), datetime(2000, 1, 2, 12, 0)),
        (34.0, "Barry", date(2000, 3, 1), datetime(2000, 1, 3, 12, 0)),
    ],
    schema="cost double, name string, birthday date, checkin timestamp",
)

df.show()
df.write.format("hudi").options(**hudi_options).mode("overwrite").save(basePath)


# query
snapshotDF = spark.read.format("hudi").load(basePath)
snapshotDF.createOrReplaceTempView("hudi_snapshot")
spark.sql("select name, cost from hudi_snapshot where cost > 35.0").show()
spark.sql(
    """
    select _hoodie_commit_time, _hoodie_record_key, _hoodie_partition_path, name, cost
    from hudi_snapshot
    """
).show()
