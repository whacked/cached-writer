# TODO: moveme
import os
import sys

import atexit
import datetime
import time
import json
import subprocess as subp
import logging


logger = logging.getLogger()
logger.setLevel(logging.DEBUG)


class CachedWriter:

    _to_flush = []

    @classmethod
    def flush_all(cls):
        for writer in cls._to_flush:
            writer.flush()

    @property
    def first(self):
        return self.buffer[0]

    @property
    def last(self):
        if self.buffer:
            return self.buffer[-1]

    def __init__(self, path, flush_limit=100):
        self.path = path
        self.buffer = []
        self.flush_limit = flush_limit

        is_first_call = len(self.__class__._to_flush) == 0
        self.__class__._to_flush.append(self)
        if is_first_call:
            logger.info('registering atexit')
            atexit.register(self.__class__.flush_all)

    def append(self, data):
        self.buffer.append(data)
        if len(self.buffer) > self.flush_limit:
            self.flush()

    def flush(self):
        with open(self.path, 'a') as ofile:
            ofile.write('\n' + '\n'.join(
                json.dumps(rec) for rec in self.buffer))
            logger.info('flushed {} records to {}'.format(
                len(self.buffer),
                self.path,
            ))
            self.buffer = []


OUTPUT_FILE_PATH = os.environ.get('LOG_FILE', 'out.jsonl')
print('logging to {}'.format(OUTPUT_FILE_PATH))
data_buffer = CachedWriter(OUTPUT_FILE_PATH)

from flask import Flask, request
app = Flask(__name__)

@app.route('/', methods=['POST'])
def slurp():
    data_buffer.append(request.json)
    if data_buffer.last:
        pprint(data_buffer.last)

    return 'ok'

if __name__ == '__main__':

    from pprint import pprint

    try:
        app.run(port=8888)
    except KeyboardInterrupt as e:
        pass
    finally:
        print('exiting...')
