from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter

# SigNozのotel-collectorに送る
exporter = OTLPSpanExporter(endpoint="http://host.docker.internal:4317", insecure=True)

provider = TracerProvider()
processor = BatchSpanProcessor(exporter)
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)

tracer = trace.get_tracer(__name__)

# テストスパン
with tracer.start_as_current_span("test-span"):
    print("sending test span")