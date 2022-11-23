Testing gps

First we setup a fake remote
  $ mkdir B
  $ cd B
  $ git init --quiet
  $ touch file_init
  $ git add file_init
  $ git commit -m "Initial commit" --quiet
  $ cd ..

Next we crate a local repo

  $ git clone --quiet B A
  $ cd A

  $ git branch -a
  * master
    remotes/origin/HEAD -> origin/master
    remotes/origin/master

  $ touch file_A
  $ touch file_B

We create our first commit

  $ git add file_A
  $ git commit -m "commit A" --quiet

and our second commit

  $ git add file_B
  $ git commit -m "commit B" --quiet

  $ git remote -v
  origin	$TESTCASE_ROOT/B (fetch)
  origin	$TESTCASE_ROOT/B (push)

We have to be careful with commit hashes as these will change depending who or
whne the test is run. So we use the following to get rid of the commit hashes.

  $ gps ls | sed 's/[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]/COHASH/'
  0           COHASH commit A 
  1           COHASH commit B 

  $ gps rr 0
  Switched to branch 'ps/tmp/isolate'
  
    The isolate_post_checkout hook was not found!
  
    This hook is NOT required but it is strongly recommended that you set it
    up. It is executed after the temporary isolation branch has been created,
    the patch cherry-picked into it and the isolation branch checked out.
  
    It is intended to be used to further verify patch isolation by verifying
    that your code bases build succeeds and your test suite passes.
  
    You can effectively have it do whatever you want as it is just a hook.
    An exit status of 0, success, informs gps that the further isolation
    verification was successful. Any non-zero exit status will indicate failure
    and cause gps to abort.
  
    You can find more information and examples of this hook and others at
    the following.
  
    https://book.git-ps.sh/tool/hooks.html
  
  Switched to branch 'master'
  Your branch is ahead of 'origin/master' by 2 commits.
    (use "git push" to publish your local commits)
  
    The isolate_post_cleanup hook was not found! Skipping...
  
    You can find more information and examples of this hook and others at
    the following.
  
    https://book.git-ps.sh/tool/hooks.html
  
  To $TESTCASE_ROOT/B
   * [new branch]      ps/rr/commit_a -> ps/rr/commit_a
  Requesting review for branch 'ps/rr/commit_a' into 'master'
  none of the git remotes configured for this repository point to a known GitHub host. To tell gh about a new GitHub host, please use `gh auth login`
  
  Error: Execution of the request_review_post_sync hook failed - ExitStatus(1)
  

  $ gps --version
  gps 6.3.1
