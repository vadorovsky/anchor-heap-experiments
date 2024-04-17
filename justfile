spl_noop_file := "spl_noop.so"
spl_noop_url := "https://github.com/Lightprotocol/light-protocol/releases/download/spl-noop-v0.2.0/spl_noop.so"

with_account_target_dir := "./with-account/target/deploy"
without_account_target_dir := "./without-account/target/deploy"

with-account:
	test -f {{ with_account_target_dir }}/{{ spl_noop_file }} \
		|| wget {{ spl_noop_url }} -O {{ with_account_target_dir }}/{{ spl_noop_file }}
	cd with-account && anchor build

test-with-account: with-account
	cd with-account && anchor test

without-account:
	test -f {{ without_account_target_dir }}/{{ spl_noop_file }} \
		|| wget {{ spl_noop_url }} -O {{ without_account_target_dir }}/{{ spl_noop_file }}
	cd without-account && anchor build

test-without-account: without-account
	cd without-account && anchor test
