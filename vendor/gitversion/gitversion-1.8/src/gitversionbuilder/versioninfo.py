from gitversionbuilder.utils import EqualityMixin, isstring
import re


class TagInterpretation(EqualityMixin):
    def __init__(self, version_components, version_tag, is_dev_version):
        assert (isinstance(version_components, list))
        assert (all(isstring(item) for item in version_components))
        assert (isstring(version_tag))
        self.version_components = version_components
        self.version_tag = version_tag
        self.is_stable = (not is_dev_version) and self.version_tag in ["", "stable", "final"]

    def __repr__(self):
        return "%s(%r)" % (self.__class__, self.__dict__)


class VersionInfo(EqualityMixin):
    def __init__(self, git_tag_name, git_commits_since_tag, git_commit_id, git_tag_exists, modified_since_commit):
        assert (isstring(git_tag_name))
        assert (isinstance(git_commits_since_tag, int))
        assert (isstring(git_commit_id))
        assert (isinstance(git_tag_exists, bool))
        assert (isinstance(modified_since_commit, bool))
        self.git_tag_name = git_tag_name
        self.git_commits_since_tag = git_commits_since_tag
        self.git_commit_id = git_commit_id
        self.git_tag_exists = git_tag_exists
        self.modified_since_commit = modified_since_commit
        self.is_dev = modified_since_commit or (not git_tag_exists) or (git_commits_since_tag != 0)

    def interpret_tag_name(self):
        matched = re.match("^v?([0-9]+(?:\.[0-9]+)*)(?:-?((alpha|beta|rc|pre|m)[0-9]?|stable|final))?$",
                           self.git_tag_name, re.IGNORECASE)
        if matched:
            version_components = matched.group(1).split('.')
            version_tag = matched.group(2)
            if version_tag is None:
                version_tag = ""
            return TagInterpretation(version_components, version_tag, self.is_dev)
        else:
            return None

    @property
    def version_string(self):
        result = ""
        if self.git_tag_exists:
            result += self.git_tag_name
        if self.git_commits_since_tag > 0:
            if result != "":
                result += "."
            result += "dev%d+rev%s" % (self.git_commits_since_tag, self.git_commit_id)
        if self.modified_since_commit:
            result += "-modified"
        return result

    def __repr__(self):
        return "%s(%r)" % (self.__class__, self.__dict__)
