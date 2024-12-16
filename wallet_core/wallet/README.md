# Wallet configuration

Temporarily stop assuming the file has not been changed:

    git update-index --no-assume-unchanged <FILE>

Then update and commit the changes. Afterwards, protect the file and assume
the file doesn't change again:

    git update-index --assume-unchanged <FILE>
