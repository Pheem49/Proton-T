import sys, os, unittest, time, tempfile
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../src')))
from proton_t import core

class TestProtonT(unittest.TestCase):
    def test_ranking(self):
        now = time.time()
        e1 = {'score': 10, 'last_access': now}
        e2 = {'score': 10, 'last_access': now - 604800}
        self.assertGreater(core.get_score(e1, now), core.get_score(e2, now))
        
    def test_project_boost(self):
        now = time.time()
        e1 = {'score': 10, 'last_access': now, 'is_project': False}
        e2 = {'score': 10, 'last_access': now, 'is_project': True}
        # Project should have 1.2x boost
        self.assertGreater(core.get_score(e2, now), core.get_score(e1, now))
        
    def test_is_project(self):
        # Create a temporary directory structure
        with tempfile.TemporaryDirectory() as tmpdir:
            proj_dir = os.path.join(tmpdir, "my_project")
            os.mkdir(proj_dir)
            with open(os.path.join(proj_dir, "package.json"), "w") as f:
                f.write("{}")
                
            regular_dir = os.path.join(tmpdir, "photos")
            os.mkdir(regular_dir)
            with open(os.path.join(regular_dir, "pic.jpg"), "w") as f:
                f.write("data")
                
            self.assertTrue(core.is_project(proj_dir))
            self.assertFalse(core.is_project(regular_dir))

    def test_parse_intent(self):
        intent1 = core.parse_intent(["recent", "project", "backend"])
        self.assertTrue(intent1['recent'])
        self.assertTrue(intent1['project'])
        # Tag map should expand 'backend'
        self.assertIn('api', intent1['tags'])
        self.assertIn('node', intent1['tags'])
        self.assertEqual(len(intent1['kws']), 0)
        
        intent2 = core.parse_intent(["today", "app", "customstring"])
        self.assertTrue(intent2['recent'])
        self.assertTrue(intent2['project'])
        self.assertEqual(intent2['kws'], ["customstring"])

    def test_match_with_intent(self):
        now = time.time()
        entry = {'score': 1, 'last_access': now, 'is_project': True}
        
        # Test 1: Project intent matches a project entry
        intent = core.parse_intent(["project", "api"])
        matched, score = core.match_with_intent("/code/my-api", entry, intent, ["project", "api"], now)
        self.assertTrue(matched)
        self.assertGreater(score, 0)
        
        # Test 2: Project intent rejects non-project entry
        non_proj_entry = {'score': 1, 'last_access': now, 'is_project': False}
        matched, score = core.match_with_intent("/code/my-api", non_proj_entry, intent, ["project", "api"], now)
        self.assertFalse(matched)
        
        # Test 3: Tag intent match
        intent_frontend = core.parse_intent(["frontend"])
        matched, score = core.match_with_intent("/code/react-app", entry, intent_frontend, ["frontend"], now)
        self.assertTrue(matched)

if __name__ == '__main__':
    unittest.main()
