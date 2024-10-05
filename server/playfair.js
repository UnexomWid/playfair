import crypto from 'node:crypto';

// These are the default constants; you should definitely change them in your own implementation

// This must be the same as the key used for packing via Playfair CLI
const PLAYFAIR_KEY = 'fair';

// These must match the values from Playfair Core
const PLAYFAIR_HMAC_KEY = 'FAIR';
const PLAYFAIR_USER_AGENT = 'Mozilla/5.0 (Windows; Windows NT 6.2;) AppleWebKit/601.34 (KHTML, like Gecko) Chrome/52.0.2897.196 Safari/535.0 Edge/13.33431';

function xor(what, key) {
    let result = Buffer.from(what);

    for (let i = 0; i < what.length; ++i) {
        result[i] = what[i] ^ key[i % key.length];
    }

    return result;
}

export default (headers) => {
    try {
        if (!headers.hasOwnProperty('user-agent') || !headers.hasOwnProperty('x-cache')) {
            return null;
        }

        if (headers['user-agent'] != PLAYFAIR_USER_AGENT) {
            return null;
        }

        const nonce = headers['x-cache'];

        if (nonce.length != 32) {
            return null;
        }

        const nonceBytes = Buffer.from(nonce, 'hex');
        const keyBytes = Buffer.from(PLAYFAIR_KEY);

        const encodedKey = xor(keyBytes, nonceBytes);
        const hash = crypto.createHmac('sha256', PLAYFAIR_HMAC_KEY).update(nonceBytes).digest();

        // <encoded_key><hash>
        return encodedKey.toString('hex').toUpperCase() + hash.toString('hex').toUpperCase();
    } catch (ex) {
        console.log(ex);
        return null;
    }
}