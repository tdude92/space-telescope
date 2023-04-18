poetry install

export PYSPARK_VENV_FILE=pyspark_venv.tar.gz
export PYSPARK_PYTHON=$(which python3)

poetry run venv-pack -o $PYSPARK_VENV_FILE
poetry run spark-submit \
    --archives $PYSPARK_VENV_FILE \
    --packages org.apache.hudi:hudi-spark3.3-bundle_2.12:0.13.0 \
    --conf 'spark.serializer=org.apache.spark.serializer.KryoSerializer' \
    --conf 'spark.sql.catalog.spark_catalog=org.apache.spark.sql.hudi.catalog.HoodieCatalog' \
    --conf 'spark.sql.extensions=org.apache.spark.sql.hudi.HoodieSparkSessionExtension' \
    "${@:1}"
