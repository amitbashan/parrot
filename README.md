# Parrot
Parrot is a simple collaborative text editing protocol.

## Usage
Synopsis for running a server instance:
```
Usage: parrot [OPTIONS] --address <ADDRESS>

Options:
  -a, --address <ADDRESS>  
  -p, --project <PROJECT>  [default: current working directory]
  -h, --help               Print help
```

## Protocol
### Request
- `Fetch`
    - `ProjectTree` - retrieves the project file tree
    - `Document(path)` - retrieves the document that resides at `path` relative to the project path
- `Update`
    - `Commit { document_path, insertions, deletions }` - commits changes (insertions/deletions) to the document that resides at `document_path`

### Response
- `Acknowledge` - server indicating acknowledgement (success status) for a certain modification submitted by a client
- `Document` - a document requested by a client
- `ProjectTree(tree)` - the project's file tree to allow a client to request files within the project