# Pipeman
pipeman用于快速验证部署基于uswift的ustack。

# 构建
使用cargo命令编译，并生成可用的二进制文件。
```bash
cargo build --release 
```

# pipeman的使用
## 基本使用
可以直接使用`pipeman --help`查看基本的命令。
```bash
pipeman --help 
Ustack deploy tool

Usage: pipeman <COMMAND>

Commands:
  deploy  Deploy Ustack
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## ustack部署
使用`pipeman help deploy`查看部署命令的基本信息。
```bash
pipeman help deploy
Deploy Ustack

Usage: pipeman deploy [OPTIONS] --arch <ARCH> --mode <MODE>

Options:
  -a, --arch <ARCH>
          The architecture of deployment

          Possible values:
          - amd64: x86_64
          - arm64: aarch64

  -m, --mode <MODE>
          The mode of deployment

          Possible values:
          - all-in-one: Single Node
          - multi-node: Cluster

  -f, --file <FILE>
          The config file of deployment

      --hosts <HOSTS>
          The ip of hosts to deploy, separated by ','

  -q, --quiet
          disable stdout log, default is false

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

可以看到deploy支持多个参数配置。
* -f, --file <FILE>。用于指定配置文件的位置，非必须，不指定时会按照顺序加载`/opt/pipeman/configs/config.toml`或者当前目录下`./configs/config.toml`的文件。
* -a, --arch <ARCH>。用于指定测试部署的架构，必须，可选值amd64和arm64。
* -m, --mode <MODE>。用于指定测试部署的模式，分为单节点和多节点（3节点），必须，可选值all-in-one和multi-node。
* --hosts <HOSTS>。用于指定测试部署的机器的IP地址，用','隔开，第一个IP是部署节点。
* -q, --quiet。用于指定是否将日志输出到stdout上，默认是否。（程序默认会将日志输入到stdout和日志文件中）

示例，如果想测试部署一个单节点的arm64集群，可以使用如下命令。
```bash
pipeman deploy -a arm64 -m all-in-one
```

示例，如果想指定hosts部署一个多节点的amd64集群，可以使用如下命令。
```bash
pipeman deploy -a arm64 -m multi-node --hosts 172.168.0.10,172.168.0.20,172.168.0.30
```

# 原理
pipeman测试部署的基本流程如下。
1. 程序检索配置文件，获取配置文件的基本信息。
2. 根据配置文件判断。
   * 是否有image_id，存在image_id表示已经上传过镜像无需上传；不存在则需要上传ISO镜像，程序会根据image_name，寻找`/opt/pipeman`下的ISO文件并上传至OpenStack。
   * 是否有volume_snapshot_id，存在volume_snapshot_id表示卷快照已经创建，程序直接利用volume_snapshot_id创建实例；不存在程序则会先创建一个基于ISO的实例，然后给实例绑定卷，再上传ignition文件，然后执行uswift-installer的安装命令，最后将卷设置为可启动，再利用卷创建卷快照。当执行完创建卷快照的步骤时，可以将得到的ID填写到配置文件中。
3. 程序根据配置文件获取到volume_snapshot_id，创建实例后，执行部署操作。
