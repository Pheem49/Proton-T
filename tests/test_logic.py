import sys, os, unittest, time
sys.path.append(os.path.join(os.path.dirname(__file__), '../src'))
from proton_t import core

class TestProtonT(unittest.TestCase):
    def test_ranking(self):
        now = time.time()
        e1 = {'score': 10, 'last_access': now}
        e2 = {'score': 10, 'last_access': now - 604800}
        self.assertGreater(core.get_score(e1, now), core.get_score(e2, now))

if __name__ == '__main__': unittest.main()
