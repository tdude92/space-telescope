from datetime import date, datetime

from pyspark import SparkConf, SparkContext
from pyspark.sql import SparkSession

spark = (
    SparkSession.builder.config(
        "spark.jars.packages", "org.apache.hudi:hudi-spark3.3-bundle_2.12:0.13.0"
    )
    .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer")
    .config(
        "spark.sql.catalog.spark_catalog",
        "org.apache.spark.sql.hudi.catalog.HoodieCatalog",
    )
    .config(
        "spark.sql.extensions", "org.apache.spark.sql.hudi.HoodieSparkSessionExtension"
    )
    .getOrCreate()
)

tableName = "hudi_trips_cow"
basePath = "file:///tmp/hudi_trips_cow"

hudi_options = {
    "hoodie.table.name": tableName,
    "hoodie.datasource.write.recordkey.field": "b",
    "hoodie.datasource.write.partitionpath.field": "d",
    "hoodie.datasource.write.table.name": tableName,
    "hoodie.datasource.write.operation": "upsert",
    "hoodie.datasource.write.precombine.field": "e",
    "hoodie.upsert.shuffle.parallelism": 2,
    "hoodie.insert.shuffle.parallelism": 2,
}

df = spark.createDataFrame(
    [
        (1, 2.0, "string1", date(2000, 1, 1), datetime(2000, 1, 1, 12, 0)),
        (2, 3.0, "string2", date(2000, 2, 1), datetime(2000, 1, 2, 12, 0)),
        (3, 4.0, "string3", date(2000, 3, 1), datetime(2000, 1, 3, 12, 0)),
    ],
    schema="a long, b double, c string, d date, e timestamp",
)

df.show()
df.write.format("hudi").options(**hudi_options).mode("overwrite").save(basePath)