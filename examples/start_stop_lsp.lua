-- A quick script that can be ran in neovim with `:%lua` to add commands to
-- interact with the hurl-language-server built with `cargo build`
-- HurlLspStart - start the built hurl-language-server
-- HurlLspStop - stop the built hurl-language-server
-- HurlLspRestart - restart the built hurl-language-server

---Start the hurl language server if not already started. If a buffer number
---is supplied this will automatically attach the buffer to the lsp.
---@param bufnr integer|nil
local function start_hurl_ls(bufnr)
	local client_capabilities = vim.lsp.protocol.make_client_capabilities()
	local resolve_properties = client_capabilities.textDocument.completion.completionItem.resolveSupport.properties
	table.insert(resolve_properties, "documentation")
	local args = {}
	if bufnr ~= nil and type(bufnr) == "number" then
		args["bufnr"] = bufnr
	else
		args["attach"] = false
	end
	vim.lsp.start({
		name = "local-hurl-ls",
		root_dir = vim.fn.getcwd(),
		capabilities = client_capabilities,
		--TODO make path work for different builds
		cmd = { vim.fn.getcwd() .. "/target/debug/hurl-language-server.exe" },
	}, args)
end

---Attach any open hurl buffers to the hurl language server
local function attach_open_hurl_buffers()
	vim.print("attaching to open hurl buffers")
	for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
		if vim.bo[bufnr].filetype == "hurl" then
			start_hurl_ls(bufnr)
		end
	end
end

local attach_lsp_group_name = "LspAttachHurlLs"

---Start the autocmd that attaches hurl_ls to hurl buffers
local function start_attach_autocmd()
	vim.api.nvim_create_autocmd("FileType", {
		pattern = { "hurl" },
		callback = function(opt)
			start_hurl_ls(opt.buf)
		end,
		group = vim.api.nvim_create_augroup(attach_lsp_group_name, { clear = true }),
		desc = "Start hurl_ls if not running and attach the hurl buffer to it.",
	})
end

---Stop the hurl language server
local function stop_hurl_ls()
	---@type vim.lsp.Client[]
	local clients = vim.lsp.get_clients({
		name = "local-hurl-ls",
	})
	if #clients > 0 then
		local client = clients[1]
		vim.lsp.stop_client(client.id, true)
	end
end

vim.api.nvim_create_user_command("HurlLspStart", function(_)
	start_hurl_ls()
	attach_open_hurl_buffers()
	start_attach_autocmd()
end, {
	desc = "Start Hurl Language Server",
})

vim.api.nvim_create_user_command("HurlLspStop", function(_)
	--Clear autocmd
	vim.api.nvim_create_augroup(attach_lsp_group_name, { clear = true })
	stop_hurl_ls()
end, {
	desc = "Stop Hurl Language Server",
})

vim.api.nvim_create_user_command("HurlLspRestart", function(_)
	---@type vim.lsp.Client[]
	local clients = vim.lsp.get_clients({
		name = "local-hurl-ls",
	})

	if #clients <= 0 then
		vim.print("hurl_ls wasn't running. Starting now...")
		start_hurl_ls()
		attach_open_hurl_buffers()
		start_attach_autocmd()
		return
	end

	local client = clients[1]
	vim.api.nvim_create_augroup(attach_lsp_group_name, { clear = true })
	vim.lsp.stop_client(client.id, true)
	local retry_count = 5
	local function schedule_restart()
		vim.print("Trying to restart hurl_ls")
		if retry_count <= 0 then
			vim.print("Failed to restart hurl_ls")
			return
		end
		if client.is_stopped() then
			start_hurl_ls()
			attach_open_hurl_buffers()
			start_attach_autocmd()
		else
			retry_count = retry_count - 1
			vim.defer_fn(function()
				schedule_restart()
			end, 100)
		end
	end
	schedule_restart()
end, {
	desc = "Restart Hurl Language Server",
})
