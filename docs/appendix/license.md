# License

Needle is released under the MIT License.

## MIT License

```
MIT License

Copyright (c) 2026 Eshan Roy <eshanized@proton.me>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## What This Means

The MIT License is a permissive open source license. You are free to:

✅ **Use** - Use Needle for any purpose, personal or commercial  
✅ **Modify** - Change the source code as you see fit  
✅ **Distribute** - Share Needle with others  
✅ **Sublicense** - Include Needle in proprietary software  
✅ **Private Use** - Use Needle privately without sharing changes

**Requirements**:
- Include the license and copyright notice when redistributing
- No warranty is provided - use at your own risk

## Third-Party Licenses

Needle depends on many open source libraries. See each dependency's license:

### Backend Dependencies

Run to see all licenses:
```bash
cd libneedle
cargo tree --prefix none | sort -u
```

Key dependencies:
- **Tokio** - MIT License
- **Axum** - MIT License
- **russh** - Apache-2.0
- **serde** - MIT OR Apache-2.0
- **hyper** - MIT License

### Frontend Dependencies

```bash
cd needleui
npm list --depth=0
```

Key dependencies:
- **Vue.js** - MIT License
- **Vite** - MIT License
- **TypeScript** - Apache-2.0

## Contributing

By contributing to Needle, you agree that your contributions will be licensed under the MIT License.

See the [Contributing guide](../developer-guide/contributing.md) for more information.

## Questions?

For licensing questions, contact:
- **Author**: Eshan Roy
- **Email**: eshanized@proton.me
