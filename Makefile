# BINDINGS_H = libnetfilter_queue/libnetfilter_queue.h stdint.h

src/nflib.rs: src/nflib.h
	bindgen $^ -o $@ --whitelist-function='nfq_open|nfq_unbind_pf'
