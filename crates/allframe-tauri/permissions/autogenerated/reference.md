## Default Permission

Default permissions for the AllFrame plugin.

Grants access to all AllFrame IPC commands:
- allframe_list: Query registered handlers
- allframe_call: Call a request/response handler
- allframe_stream: Start a streaming handler (returns stream_id, emits events)
- allframe_stream_cancel: Cancel an active stream

#### This default permission set includes the following:

- `allow-allframe-list`
- `allow-allframe-call`
- `allow-allframe-stream`
- `allow-allframe-stream-cancel`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`allframe-tauri:allow-allframe-call`

</td>
<td>

Enables the allframe_call command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:deny-allframe-call`

</td>
<td>

Denies the allframe_call command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:allow-allframe-list`

</td>
<td>

Enables the allframe_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:deny-allframe-list`

</td>
<td>

Denies the allframe_list command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:allow-allframe-stream`

</td>
<td>

Enables the allframe_stream command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:deny-allframe-stream`

</td>
<td>

Denies the allframe_stream command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:allow-allframe-stream-cancel`

</td>
<td>

Enables the allframe_stream_cancel command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`allframe-tauri:deny-allframe-stream-cancel`

</td>
<td>

Denies the allframe_stream_cancel command without any pre-configured scope.

</td>
</tr>
</table>
