'use strict';

const { HTMLRewriter } = require('lol-html');

const chunks = [];
const rewriter = new HTMLRewriter('utf8', (chunk) => {
  chunks.push(chunk);
});

const endTags = [];
rewriter.on('a[href]', {
  element(el) {
    const href = el
      .getAttribute('href')
      .replace('http:', 'https:');
    el.setAttribute('href', href);
    el.setAttribute("start", 'byte '+el.sourceLocationBytes[0]);

    el.onEndTag((tag) => {
      endTags.push(tag.name);
    });
  },
});

[
  '<div><a href=',
  'http://example.com>',
  '</a></div>',
].forEach((part) => {
  rewriter.write(Buffer.from(part));
});

rewriter.end();

const output = Buffer.concat(chunks).toString('utf8');
if (output != '<div><a href="https://example.com" start="byte 5"></a></div>') {
  throw "fail\ngot: " + output;
}

if (endTags.length != 1 || endTags[0] != 'a') {
  throw "onEndTag fail";
}
