define TYPE_PAIRS
CStreamingHandlerSink:streaming_sink_t \
CStreamingHandler:streaming_handler_t \
Iter<lol_html_attribute_t>:attributes_iterator_t \
AttributesIterator:attributes_iterator_t \
Attribute:attribute_t \
Comment:comment_t \
Doctype:doctype_t \
DocumentEnd:doc_end_t \
Element:element_t \
EndTag:end_tag_t \
HtmlRewriterBuilder:rewriter_builder_t \
HtmlRewriter:rewriter_t \
Selector:selector_t \
TextChunk:text_chunk_t \
Str:str_t
endef


define create_sed_expr
-e 's/(struct )?lol_html_$(firstword $(subst :, ,$1))/lol_html_$(lastword $(subst :, ,$1))/g' \
-e 's/struct lol_html_$(lastword $(subst :, ,$1))/struct lol_html_$(firstword $(subst :, ,$1))/g'
endef

SED_EXPRS := $(foreach pair,$(TYPE_PAIRS),$(call create_sed_expr,$(pair)))

all::
	cargo expand > /tmp/lol_html_h.rs
	cbindgen -c cbindgen.toml /tmp/lol_html_h.rs | sed -E $(SED_EXPRS) -e 's#///#// #g' > include/lol_html.h
	git --no-pager diff -w -- include/

