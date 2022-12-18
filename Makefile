
build:
	anchor build && npm run build
.PHONY: build

test:
	make build && anchor test
.PHONY: test

deploy:
	make build && anchor deploy --provider.cluster devnet
.PHONY: deploy

deploy-idl:
	make build && anchor idl init -f target/idl/lucifer.json DAmaWox5y8sKNMJE34pp6atKBhwk2SKXgDtSiv6gHjJL --provider.cluster devnet
.PHONY: deploy-idl

upgrade-idl:
	make build && anchor idl upgrade DAmaWox5y8sKNMJE34pp6atKBhwk2SKXgDtSiv6gHjJL -f target/idl/lucifer.json --provider.cluster devnet
.PHONY: deploy-idl