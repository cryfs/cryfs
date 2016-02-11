import unittest

from gitversionbuilder.versioninfo import VersionInfo, TagInterpretation


class VersionInfoTest(unittest.TestCase):
    def test_equals(self):
        self.assertEqual(VersionInfo("v1.6.0", 20, "23fa", True, False),
                         VersionInfo("v1.6.0", 20, "23fa", True, False))

    def test_not_equals_tag(self):
        self.assertNotEqual(VersionInfo("v1.6.0", 20, "23fa", True, False),
                            VersionInfo("v1.6.1", 20, "23fa", True, False))

    def test_not_equals_commits_since_tag(self):
        self.assertNotEqual(VersionInfo("v1.6.1", 20, "23fa", True, False),
                            VersionInfo("v1.6.1", 21, "23fa", True, False))

    def test_not_equals_commit_id(self):
        self.assertNotEqual(VersionInfo("v1.6.1", 20, "23fa", True, False),
                            VersionInfo("v1.6.1", 20, "23fb", True, False))

    def test_not_equals_is_tag(self):
        self.assertNotEqual(VersionInfo("v1.6.1", 20, "23fa", True, False),
                            VersionInfo("v1.6.1", 20, "23fa", False, False))

    def test_not_equals_modified_since_commit(self):
        self.assertNotEqual(VersionInfo("v1.6.1", 20, "23fa", True, False),
                            VersionInfo("v1.6.1", 20, "23fa", True, True))

    def test_version_string_for_tag(self):
        self.assertEqual("v1.5", VersionInfo("v1.5", 0, "23fa", True, False).version_string)

    def test_version_string_for_tag_modified(self):
        self.assertEqual("v1.5-modified", VersionInfo("v1.5", 0, "23fa", True, True).version_string)

    def test_version_string_with_no_tag(self):
        self.assertEqual("dev2+rev23fa", VersionInfo("develop", 2, "23fa", False, False).version_string)

    def test_version_string_with_no_tag_modified(self):
        self.assertEqual("dev2+rev23fa-modified", VersionInfo("develop", 2, "23fa", False, True).version_string)

    def test_version_string_with_commits_since_tag(self):
        self.assertEqual("v1.5.dev2+rev23fa", VersionInfo("v1.5", 2, "23fa", True, False).version_string)

    def test_version_string_with_commits_since_tag_modified(self):
        self.assertEqual("v1.5.dev2+rev23fa-modified", VersionInfo("v1.5", 2, "23fa", True, True).version_string)

    def test_is_dev_1(self):
        self.assertTrue(VersionInfo("1.0", 1, "23fa", True, False).is_dev)

    def test_is_dev_123(self):
        self.assertTrue(VersionInfo("1.0", 123, "23fa", True, False).is_dev)

    def test_is_dev_no_commits(self):
        self.assertTrue(VersionInfo("1.0", 0, "23fa", False, False).is_dev)

    def test_is_dev_modified(self):
        self.assertTrue(VersionInfo("1.0", 0, "23fa", True, True).is_dev)

    def test_is_not_dev(self):
        self.assertFalse(VersionInfo("1.0", 0, "23fa", True, False).is_dev)

    def test_interpret_valid_tag_name(self):
        self.assertEqual(TagInterpretation(["1"], "", False),
                         VersionInfo("1", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_plain(self):
        self.assertEqual(TagInterpretation(["1", "0"], "", False),
                         VersionInfo("1.0", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_alpha(self):
        self.assertEqual(TagInterpretation(["1", "0"], "alpha", False),
                         VersionInfo("1.0alpha", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_alpha_number(self):
        self.assertEqual(TagInterpretation(["1", "0"], "alpha2", False),
                         VersionInfo("1.0alpha2", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_beta(self):
        self.assertEqual(TagInterpretation(["1", "0"], "beta", False),
                         VersionInfo("1.0beta", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_beta_number(self):
        self.assertEqual(TagInterpretation(["1", "0"], "beta3", False),
                         VersionInfo("1.0beta3", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_with_dash(self):
        self.assertEqual(TagInterpretation(["1", "02", "3"], "beta", False),
                         VersionInfo("1.02.3-beta", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_with_zeroes_in_component(self):
        self.assertEqual(TagInterpretation(["1", "020", "3"], "beta", False),
                         VersionInfo("1.020.3-beta", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_stable(self):
        self.assertEqual(TagInterpretation(["1", "02"], "stable", False),
                         VersionInfo("1.02-stable", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_final(self):
        self.assertEqual(TagInterpretation(["0", "8"], "final", False),
                         VersionInfo("0.8final", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_M3(self):
        self.assertEqual(TagInterpretation(["0", "8"], "M3", False),
                         VersionInfo("0.8-M3", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_m3(self):
        self.assertEqual(TagInterpretation(["0", "8"], "m3", False),
                         VersionInfo("0.8m3", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_rc2(self):
        self.assertEqual(TagInterpretation(["0", "8"], "rc2", False),
                         VersionInfo("0.8rc2", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_RC2(self):
        self.assertEqual(TagInterpretation(["0", "8"], "RC2", False),
                         VersionInfo("0.8-RC2", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_pre2(self):
        self.assertEqual(TagInterpretation(["0", "8"], "pre2", False),
                         VersionInfo("0.8-pre2", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_of_dev_version_1(self):
        self.assertEqual(TagInterpretation(["0", "8"], "", True),
                         VersionInfo("0.8", 1, "23fa", True, False).interpret_tag_name())

    def test_interpret_valid_tag_name_of_dev_version_2(self):
        self.assertEqual(TagInterpretation(["0", "8"], "", True),
                         VersionInfo("0.8", 123, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name(self):
        self.assertEqual(None, VersionInfo("develop", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name_invalid_tag(self):
        self.assertEqual(None, VersionInfo("1.0invalid", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name_invalid_tag_with_dash(self):
        self.assertEqual(None, VersionInfo("1.0-invalid", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name_invalid_number(self):
        self.assertEqual(None, VersionInfo("develop-alpha", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name_invalid_component_separator(self):
        self.assertEqual(None, VersionInfo("1,0-alpha", 0, "23fa", True, False).interpret_tag_name())

    def test_interpret_invalid_tag_name_invalid_missing_component(self):
        self.assertEqual(None, VersionInfo("1,-alpha", 0, "23fa", True, False).interpret_tag_name())


class TagInterpretationTest(unittest.TestCase):
    def test_equals(self):
        self.assertEqual(TagInterpretation(["1", "2"], "alpha", False),
                         TagInterpretation(["1", "2"], "alpha", False))

    def test_not_equals_version_tag(self):
        self.assertNotEqual(TagInterpretation(["1", "2"], "beta", False),
                            TagInterpretation(["1", "2"], "alpha", False))

    def test_not_equals_components_1(self):
        self.assertNotEqual(TagInterpretation(["1"], "alpha", False),
                            TagInterpretation(["1", "2"], "alpha", False))

    def test_not_equals_components_2(self):
        self.assertNotEqual(TagInterpretation(["1", "3"], "alpha", False),
                            TagInterpretation(["1", "2"], "alpha", False))

    def test_alpha_is_not_stable(self):
        self.assertFalse(TagInterpretation(["1"], "alpha", False).is_stable)

    def test_beta_is_not_stable(self):
        self.assertFalse(TagInterpretation(["1"], "beta", False).is_stable)

    def test_rc3_is_not_stable(self):
        self.assertFalse(TagInterpretation(["1"], "rc3", False).is_stable)

    def test_M3_is_not_stable(self):
        self.assertFalse(TagInterpretation(["1"], "M3", False).is_stable)

    def test_stable_is_stable(self):
        self.assertTrue(TagInterpretation(["1"], "stable", False).is_stable)

    def test_final_is_stable(self):
        self.assertTrue(TagInterpretation(["1"], "final", False).is_stable)

    def test_plain_is_stable(self):
        self.assertTrue(TagInterpretation(["1"], "", False).is_stable)

    def test_dev_is_not_stable(self):
        self.assertFalse(TagInterpretation(["1"], "", True).is_stable)


if __name__ == '__main__':
    unittest.main()
