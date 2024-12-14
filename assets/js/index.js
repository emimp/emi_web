
let width = window.innerWidth;
let height = window.innerHeight;

let canvas = document.getElementById('tuiCanvas');
canvas.width = width;
canvas.height = height;

const ctx = canvas.getContext('2d');
ctx.font = `25px monospace`;

ctx.textBaseline = 'top';
document.addEventListener('keydown', (event) => {
  const keyPress = {
      key: event.key, // This captures the key pressed (e.g., 'ArrowRight')
  };
  if (socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(keyPress)); // Send the keyPress object as a JSON string
      console.log(keyPress)
  }
});

// Establish WebSocket connection
const socket = new WebSocket('ws://192.168.1.213:3001/ws');

socket.onopen = () => {
  console.log('Connected to WebSocket');
  let c = calculateMaxChars(canvas, fontSize = 30, fontFamily = 'monospace')
  console.log("MAX: ", c);
  const canvasResolution = {
		width: c.maxCharsX,
		height: c.maxLinesY,
	};
	socket.send(JSON.stringify(canvasResolution));
};

socket.onmessage = function(event) {
	// console.log("Raw data from WebSocket:", event.data); // Log raw data for debugging
	try {
	  // Parse the incoming data as JSON
	  const data = JSON.parse(JSON.parse(event.data));
	  let area = data.area;
	  let content = data.content
	  renderTUI(area, content)
  
	} catch (error) {
	  console.error('Error parsing data:', error); // Log any parsing errors
	}
  };
  

socket.onerror = (error) => {
  console.error('WebSocket error:', error);
};

socket.onclose = () => {
  console.log('Disconnected from WebSocket');
};


function renderTUI(area, content) {
	ctx.clearRect(0, 0, canvas.width, canvas.height);
	ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
	ctx.fillRect(0, 0, canvas.width, canvas.height);
  
	const lineHeight = 32; // Adjust based on font size
	for (let y = 0; y < content.length; y++) {
	  // Remove quotation marks from the text
	  const text = content[y].replace(/['"]/g, '');
	  const x = area.x * 10; // Scale for visual positioning
	  const posY = area.y * 10 + y * lineHeight;
  
	  ctx.fillStyle = '#d4d4d4';
  
	  ctx.fillText(text, x, posY);
	}
  }
  
  function calculateMaxChars(canvas, fontSize, fontFamily) {
    const ctx = canvas.getContext('2d');

    const charWidth = ctx.measureText("W").width;
    const maxCharsX = Math.floor(canvas.width / charWidth);
    const lineHeight = fontSize * 1.05;
    const maxLinesY = Math.floor(canvas.height / lineHeight);

    return {
        maxCharsX,
        maxLinesY
    };
}

(function() {

    "use strict";

    var $body = document.querySelector('body');

    // Methods/polyfills.

    // classList | (c) @remy | github.com/remy/polyfills | rem.mit-license.org
    !function() {
        function t(t) {
            this.el = t;
            for (var n = t.className.replace(/^\s+|\s+$/g, "").split(/\s+/), i = 0; i < n.length; i++) e.call(this, n[i]);
        }
        function n(t, n, i) {
            Object.defineProperty ? Object.defineProperty(t, n, { get: i }) : t.__defineGetter__(n, i);
        }

        if (!("undefined" == typeof window.Element || "classList" in document.documentElement)) {
            var i = Array.prototype, e = i.push, s = i.splice, o = i.join;
            t.prototype = {
                add: function(t) {
                    this.contains(t) || (e.call(this, t), this.el.className = this.toString());
                },
                contains: function(t) {
                    return -1 != this.el.className.indexOf(t);
                },
                item: function(t) {
                    return this[t] || null;
                },
                remove: function(t) {
                    if (this.contains(t)) {
                        for (var n = 0; n < this.length && this[n] != t; n++);
                        s.call(this, n, 1), this.el.className = this.toString();
                    }
                },
                toString: function() {
                    return o.call(this, " ");
                },
                toggle: function(t) {
                    return this.contains(t) ? this.remove(t) : this.add(t), this.contains(t);
                }
            };
            window.DOMTokenList = t;
            n(Element.prototype, "classList", function() {
                return new t(this);
            });
        }
    }();

    // canUse
    window.canUse = function(p) {
        if (!window._canUse) window._canUse = document.createElement("div");
        var e = window._canUse.style, up = p.charAt(0).toUpperCase() + p.slice(1);
        return p in e || "Moz" + up in e || "Webkit" + up in e || "O" + up in e || "ms" + up in e;
    };

    // window.addEventListener polyfill
    (function() {
        if ("addEventListener" in window) return;
        window.addEventListener = function(type, f) {
            window.attachEvent("on" + type, f);
        };
    })();

    // Play initial animations on page load.
    window.addEventListener('load', function() {
        window.setTimeout(function() {
            $body.classList.remove('is-preload');
        }, 100);
    });

    // Slideshow Background.
    (function() {

        // Settings.
        var settings = {
            // Images (in the format of 'url': 'alignment').
            images: {
                'images/bg01.jpg': 'center',
                'images/bg02.jpg': 'center',
                'images/bg03.jpg': 'center'
            },
            // Delay.
            delay: 6000
        };

        // Vars.
        var pos = 0, lastPos = 0, $wrapper, $bgs = [], $bg, k, v;

        // Create BG wrapper, BGs.
        $wrapper = document.createElement('div');
        $wrapper.id = 'bg';
        $body.appendChild($wrapper);

        for (k in settings.images) {
            // Create BG.
            $bg = document.createElement('div');
            $bg.style.backgroundImage = 'url("' + k + '")';
            $bg.style.backgroundPosition = settings.images[k];
            $wrapper.appendChild($bg);

            // Add it to array.
            $bgs.push($bg);
        }

        // Main loop.
        $bgs[pos].classList.add('visible');
        $bgs[pos].classList.add('top');

        // Bail if we only have a single BG or the client doesn't support transitions.
        if ($bgs.length == 1 || !canUse('transition')) return;

        window.setInterval(function() {

            lastPos = pos;
            pos++;

            // Wrap to beginning if necessary.
            if (pos >= $bgs.length) pos = 0;

            // Swap top images.
            $bgs[lastPos].classList.remove('top');
            $bgs[pos].classList.add('visible');
            $bgs[pos].classList.add('top');

            // Hide last image after a short delay.
            window.setTimeout(function() {
                $bgs[lastPos].classList.remove('visible');
            }, settings.delay / 2);

        }, settings.delay);

    })();

})();
