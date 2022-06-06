from flask import Flask, request
from flask_restful import Resource, Api
import json


def load_channels(filename):
    with open(filename, "r") as f:
        return json.load(f)


def add_score(info):
    db.append({"id": db[len(db) - 1]["id"] + 1, "score": info["score"]})
    save_db()
    return "Score added"


def save_db():
    with open("database.json", "w") as f:
        json.dump(db, f)


class Scores(Resource):
    def get(self):
        return [{"id": c["id"], "score": c["score"]} for c in db]

    def post(self):
        return add_score(request.get_json())


app = Flask(__name__)
api = Api(app)
db = load_channels("database.json")
api.add_resource(Scores, '/scores')

if __name__ == "__main__":
    app.run(debug=True)
