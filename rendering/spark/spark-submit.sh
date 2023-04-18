PYSPARK_VENV_FILE=pyspark_venv.tar.gz

poetry install
poetry run venv-pack -o $PYSPARK_VENV_FILE
poetry run spark-submit --archives $PYSPARK_VENV_FILE "${@:1}"
